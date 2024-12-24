use crate::common::*;


#[async_trait]
pub trait QueryService {

}

#[derive(Debug, new)]
pub struct QueryServicePub {}


#[async_trait]
impl QueryService for QueryServicePub {



}