use super::{Flash, FlashMessage};
use actix_session::Session;
use actix_web::dev::{
    self, Extensions, Payload, Service, ServiceRequest, ServiceResponse, Transform,
};
use actix_web::{web::Data, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Client context passed to routes.
#[derive(Debug)]
pub struct Context {
    /// User data. Optional. None is a guest user.
    //pub client: Option<Profile>,
    /// List of user group ids. Guests may receive unregistered/portal roles.
    //pub groups: Vec<i32>,
    /// Permission data.
    //pub permissions: Data<PermissionData>,
    /// Flash messages.
    pub messages: Vec<FlashMessage>,
    /// Randomly generated string for CSR.
    pub nonce: String,
    /// Time the request started for page load statistics.
    pub request_start: Instant,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            // Guests and users.
            //permissions: Data::new(PermissionData::default()),
            //groups: Vec::new(),
            // Only users.
            //client: None,
            // Generally left default.
            messages: Default::default(),
            nonce: Self::nonce(),
            request_start: Instant::now(),
        }
    }
}

impl Context {
    //pub async fn from_session(session: &Session, permissions: Data<PermissionData>) -> Self {
    //    use crate::group::get_group_ids_for_client;
    //    use crate::session::authenticate_client_by_session;
    //
    //    let db = get_db_pool();
    //    let client = authenticate_client_by_session(session).await;
    //    let groups = get_group_ids_for_client(db, &client).await;
    //
    //    ClientCtxInner {
    //        client,
    //        groups,
    //        permissions,
    //        ..Default::default()
    //    }
    //}

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

        // Hash: Timestamp
        use std::time::{SystemTime, UNIX_EPOCH};
        hasher.update(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System clock before 1970. Really?")
                .as_millis()
                .to_ne_bytes(),
        );

        hasher.finalize().to_string()
    }

    pub fn flash(&mut self, class: Flash, message: &str) {
        self.messages.push(FlashMessage {
            class,
            message: message.to_owned(),
        })
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
        ready(Ok(Context::default()))
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
        let session = Session::extract(&httpreq).into_inner();
        let req = ServiceRequest::from_parts(httpreq, payload);

        // If we do not have permission data there is no client interface to access.
        Box::pin(async move {
            //if let Some(perm_arc) = req.app_data::<Data<PermissionData>>() {
            //    let perm_arc = perm_arc.clone();
            //
            //    match session {
            //        Ok(session) => req.extensions_mut().insert(Data::new(
            //            ClientCtxInner::from_session(&session, perm_arc).await,
            //        )),
            //        Err(err) => {
            //            log::error!("Unable to extract Session data in middleware: {}", err);
            //            None
            //        }
            //    };
            //};

            svc.call(req).await
        })
    }
}
