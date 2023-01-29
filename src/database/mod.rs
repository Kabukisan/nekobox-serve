use crate::database::redis::open_task_db_connection;
use crate::error::Error;
use crate::models::{TaskRequest, TaskResponse};
use ::redis::Commands;

pub(crate) mod models;
pub(crate) mod redis;
pub(crate) mod sqlite;

pub fn set_task_response(task: &TaskResponse) -> Result<(), Error> {
    let mut connection = open_task_db_connection();
    let json_string = serde_json::to_string(&task).unwrap();
    connection.set::<&str, String, String>(&task.status_id, json_string)?;
    Ok(())
}

pub fn get_task_response(task: &TaskRequest) -> Result<TaskResponse, Error> {
    let mut connection = open_task_db_connection();
    let json_string = connection.get::<&str, String>(&task.status_id).unwrap();
    Ok(serde_json::from_str(&json_string)?)
}

pub fn del_task_response(task: &TaskRequest) -> Result<(), Error> {
    let mut connection = open_task_db_connection();
    connection.del(&task.status_id)?;
    Ok(())
}
