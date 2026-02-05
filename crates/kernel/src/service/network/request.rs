use network::errors::RequestError;

pub enum RequestResult<T> {
    Success(T),
    ClientError(T),
    ServerError(T),
    Failed(RequestError),
}
