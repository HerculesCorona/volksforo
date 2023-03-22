use super::FlashJar;
use crate::model::UserSession;
use crate::session::Visitor;
use actix_web::cookie::Cookie;
use actix_web::dev::{
    self, Extensions, Payload, Service, ServiceRequest, ServiceResponse, Transform,
};
use actix_web::{web::Data, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::LocalBoxFuture;
use scylla::Session as ScyllaSession;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Client context passed to routes.
#[derive(Debug)]
pub struct Context {
    /// List of user group ids. Guests may receive unregistered/portal roles.
    pub groups: Vec<i32>,
    /// Flash messages.
    pub jar: FlashJar,
    /// Randomly generated string for CSR.
    pub nonce: String,
    /// Permission data.
    //pub permissions: Data<PermissionData>,
    /// Time the request started for page load statistics.
    pub request_start: Instant,
    /// Visitor data.
    pub visitor: Visitor,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            // Guests and users.
            //permissions: Data::new(PermissionData::default()),
            groups: Vec::new(),
            // Only users.
            visitor: Default::default(),
            // Generally left default.
            jar: Default::default(),
            nonce: Self::nonce(),
            request_start: Instant::now(),
        }
    }
}

impl Context {
    /// Pass a Cookie to try and restore a session.
    pub async fn from_cookie(scylla: Data<ScyllaSession>, cookie: &Cookie<'_>) -> Self {
        //let groups = get_group_ids_for_client(db, &client).await;
        match Uuid::parse_str(cookie.value()) {
            Ok(uuid) => match Visitor::new_from_uuid(scylla, uuid).await {
                Ok(visitor) => {
                    log::debug!("Context::from_cookie visitor: {:?}", &visitor);
                    Self {
                        visitor,
                        ..Default::default()
                    }
                }
                Err(e) => {
                    log::debug!("Context::from_cookie auth error: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::debug!("Context::from_cookie parse error: {}", e);
                Self::default()
            }
        }
    }

    /// Removed Context from the Extensions jar.
    pub fn from_extensions(extensions: &mut Extensions) -> Self {
        match extensions.remove::<Self>() {
            // Existing record in extensions; pull it and return clone.
            Some(cbox) => cbox,
            // No existing record; create and insert it.
            None => Self::default(),
        }
    }

    /// Returns a hash unique to each request used for CSP.
    /// See: <https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/nonce>
    /// and <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>
    pub fn nonce() -> String {
        let mut hasher = blake3::Hasher::new();

        // Hash: Salt
        hasher.update(
            std::env::var("VF_SALT")
                .expect("VF_SALT is unset")
                .as_bytes(),
        );

        // Hash: Timestamp in nanoseconds
        hasher.update(&chrono::Utc::now().timestamp_nanos().to_ne_bytes());

        // Finalize.
        hasher.finalize().to_string()
    }

    /// Returns the security nonce from ContextInner.
    /// Generates once in ContextInner::default() as random.
    pub fn get_nonce(&self) -> &String {
        &self.nonce
    }

    /// Returns Duration representing request time.
    pub fn request_time(&self) -> Duration {
        Instant::now() - self.request_start
    }

    /// Returns human readable representing request time.
    pub fn request_time_as_string(&self) -> String {
        let us = self.request_time().as_micros();
        if us > 5000 {
            format!("{}ms", us / 1000)
        } else {
            format!("{}μs", us)
        }
    }
}

/// This implementation is what actually provides the `client: ClientCtx` in the parameters of route functions.
impl FromRequest for Context {
    /// The associated error which can be returned.
    type Error = Error;
    /// Future that resolves to a Self.
    type Future = Ready<Result<Self, Self::Error>>;

    /// Create a Self from request parts asynchronously.
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(Ok(Self::from_extensions(&mut req.extensions_mut())))
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for Context
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ContextMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContextMiddleware {
            service: Rc::new(service),
        }))
    }
}

/// Client context middleware
pub struct ContextMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ContextMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // Borrows of `req` must be done in a precise way to avoid conflcits. This order is important.
        let (httpreq, payload) = req.into_parts();
        //let session = ActixSession::extract(&httpreq).into_inner();
        let scylla = httpreq.app_data::<Data<ScyllaSession>>().cloned(); // Clone like this to avoid inheritence issues with next line.
        let cookie = httpreq.cookie("vf_session");
        let req = ServiceRequest::from_parts(httpreq, payload);

        // If we do not have permission data there is no client interface to access.
        Box::pin(async move {
            if let Some(cookie) = &cookie {
                if let Some(scylla) = scylla {
                    let context = Context::from_cookie(scylla.clone(), cookie).await;

                    if let Some(session_id) = &context.visitor.session_id {
                        let uuid = session_id.to_owned();
                        tokio::spawn(async move {
                            match UserSession::bump_last_seen_at(scylla.clone(), &uuid).await {
                                Ok(_) => {}
                                Err(err) => {
                                    log::error!("Failed to bump last seen at: {}", err);
                                }
                            }
                        });
                    }

                    req.extensions_mut().insert(context);
                }
            }

            svc.call(req).await
        })
    }
}
