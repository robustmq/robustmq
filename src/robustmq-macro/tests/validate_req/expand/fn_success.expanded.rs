use prost_validate::Validator;
pub struct FooService;
pub struct Request {
    #[validate(r#type(string(r#const = "foo")))]
    name: String,
}
impl ::prost_validate::Validator for Request {
    #[allow(irrefutable_let_patterns)]
    #[allow(unused_variables)]
    fn validate(&self) -> prost_validate::Result<()> {
        let name = &self.name;
        if name != "foo" {
            return Err(
                ::prost_validate::Error::new(
                    "",
                    ::prost_validate::errors::string::Error::Const("foo".to_string()),
                ),
            );
        }
        Ok(())
    }
}
pub struct Response;
pub trait HeyGRPC {
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn hey<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<Request>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<Response>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
}
impl HeyGRPC for FooService {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn hey<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<Request>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Result<tonic::Response<Response>, tonic::Status>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            {
                {
                    let ___request__validate_ref = _request.get_ref();
                    if let Err(e) = ___request__validate_ref.validate() {
                        return Err(tonic::Status::invalid_argument(e.to_string()));
                    }
                }
            }
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Result<tonic::Response<Response>, tonic::Status>,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let __self = self;
            let _request = _request;
            let __ret: Result<tonic::Response<Response>, tonic::Status> = {
                Ok(tonic::Response::new(Response {}))
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
fn main() {}
