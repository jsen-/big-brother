use crate::error::ResponseError;

pub trait ResponseToResult: k8s_openapi::Response {
    type Output;
    fn to_result(self) -> Result<Self::Output, ResponseError>;
}

#[macro_export]
macro_rules! impl_rtr {
    ($ty:ty, $target:ty) => {
        impl crate::api_resources::response_to_result::ResponseToResult for $ty {
            type Output = $target;
            #[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/5005
            fn to_result(self) -> ::std::result::Result<Self::Output, crate::error::ResponseError> {
                match self {
                    <$ty>::Ok(ret) => ::std::result::Result::Ok(ret),
                    <$ty>::Other(result) => match result {
                        ::std::result::Result::Ok(opt) => ::std::result::Result::Err(crate::error::ResponseError::Other(opt)),
                        ::std::result::Result::Err(err) => ::std::result::Result::Err(crate::error::ResponseError::Deserialize(err)),
                    },
                }
            }
        }
    };
}

#[macro_export]
macro_rules! wrap_rtr {
    ($newtype:ident, $target:ty) => {
        pub struct $newtype(k8s_openapi::$newtype);

        impl ::std::ops::Deref for $newtype {
            type Target = k8s_openapi::$newtype;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl ::std::convert::From<k8s_openapi::$newtype> for $newtype {
            fn from(src: k8s_openapi::$newtype) -> Self {
                Self(src)
            }
        }

        impl ::k8s_openapi::Response for $newtype {
            fn try_from_parts(
                status_code: ::k8s_openapi::http::StatusCode,
                buf: &[u8],
            ) -> ::std::result::Result<(Self, usize), ::k8s_openapi::ResponseError> {
                <k8s_openapi::$newtype>::try_from_parts(status_code, buf).map(|(x, len)| (x.into(), len))
            }
        }
        impl_rtr!(k8s_openapi::$newtype, $target);

        #[rustfmt::skip]
        impl crate::api_resources::response_to_result::ResponseToResult for $newtype {
            type Output = $target;
            fn to_result(self) -> ::std::result::Result<Self::Output, crate::error::ResponseError> {
                self.0.to_result()
            }
        }
    };
}
