use std::future::{ready, Ready};

// use actix_session::SessionExt;
use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    // http,
    Error,
    // HttpResponse,
};
use futures_util::future::LocalBoxFuture;
// use crate::service::Message;


pub struct CheckSession;

impl<S, B> Transform<S, ServiceRequest> for CheckSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckSessionMiddleware { service }))
    }
}
pub struct CheckSessionMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    // let user_id: Option<i64> = session.get("user_id").unwrap_or(None);

    // match user_id {
    //     Some(id) => {
    //         // keep the user's session alive
    //         session.renew();
    //         Ok(id)
    //     }
    //     None => Err(HttpResponse::Unauthorized().json("Unauthorized")),
    // }

    fn call(&self, request: ServiceRequest) -> Self::Future {
        // Change this to see the change in outcome in the browser.
        // Usually this boolean would be acquired from a password check or other auth verification.
        // let session = request.get_session();
        // log::info!("Session {:?} {:?}", session.status(), session.entries());

        // if let Some(messages) = session.get::<Vec<Message>>("messages").unwrap() {
        //     for message in messages {
        //         log::debug!("{:?}", message);
        //     }
        // } else {
        //     let messages: Vec<Message> = Vec::new();
        //     session.insert("messages", messages).unwrap();
        // }

        // if let Some(is_logged_in) = session.get::<bool>("is_logged").unwrap() {
        //     if !is_logged_in  && request.path() != "/login" {
        //         let (request, _pl) = request.into_parts();

        //         let response = HttpResponse::Found()
        //             .insert_header((http::header::LOCATION, "/login"))
        //             .finish()
        //             // constructed responses map to "right" body
        //             .map_into_right_body();
        //         log::info!("Redirection login false /login");
        //         return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        //     }
        // } else {
        //     if request.path() != "/login" {
        //         let (request, _pl) = request.into_parts();

        //         let response = HttpResponse::Found()
        //             .insert_header((http::header::LOCATION, "/login"))
        //             .finish()
        //             // constructed responses map to "right" body
        //             .map_into_right_body();
        //         log::info!("Redirect no login /login");
        //         return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        //     }
        // }

        let res = self.service.call(request);

        Box::pin(async move {
            // forwarded responses map to "left" body
            res.await.map(ServiceResponse::map_into_left_body)
        })
    }
}