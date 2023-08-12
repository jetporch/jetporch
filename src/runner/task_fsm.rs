use crate::module_base::common::*

/*

pub enum TaskRequestType {
    Validate,Query,Create,Remove,Modify,
}

pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Option<HashMap<String, String>>
}

pub enum TaskStatus {
    Validated,NeedsCreation,NeedsRemoval,NeedsModification,Done,Failed
}

pub struct TaskResponse {
    pub is: TaskStatus,pub changes: Option<HashMap<String, String>>
}

*/

// run a task on one or more hosts -- check modes (syntax/normal), or for 'real', on any connection type
pub fn run_task(context: &PlaybookContext,
    visitor: &dyn PlaybookVisitor, 
    connection_factory: &dyn ConnectionFactory, 
    task: &Task) -> Result<(), String> {

    let task_result = visitor.is_syntax_only();


    if syntax {
        let sc = run_task_on_hosts(context, visitor, connection, task);
        match sc.is {
            TaskStatus::Validated => { return Ok(()) } 
            _ => { Err(format!("resource defintion integrity check failed: {}", sc.msg )) }
        }
    }

    hosts = context.get_all_hosts();
    for host in hosts {
        let connection = connection_factory.get_connection(host);
        run_task_on_host(context, visitor, connection, task);
    }

    return Ok(());

    // FIXME: the results per host should be set on the context, and if any failures arise,
    // we know to take them *out* of the all_hosts_pool and record the failures


}

fn run_task_on_host(
    context: &PlaybookContext,
    visitor: &dyn PlaybookVisitor, 
    connection: &dyn Connection, 
    task: &Task) -> TaskResponse {

    let syntax     = visitor.is_syntax_only();
    let check_mode = visitor.is_check_mode();

    let vrc = task.dispatch(context, visitor, connection, TaskRequest { request_type: TaskRequestType::Validate, changes: None });
    match vrc.is {
        TaskStatus::Validated => { vrc },
        TaskStatus::Failed => { vrc },
        _ => { panic!("module internal fsm state invalid (on verify)"); 
    }

    if syntax_mode || vrc.is == TaskStatus::Failed {
        return vrc;
    }

    qrc = task.dispatch(context, visitor, connection, TaskRequest { request_type: TaskRequestType::Query, changes: None });
    let result = match vrc.is {
        TaskStatus::NeedsCreation => {
            match modify_mode {
                true => {
                    let crc = task.dispatch(context, visitor, connection, TaskRequest { 
                        request_type: TaskRequestType::Create, 
                        changes: None 
                    });
                    match crc.is {
                        TaskStatus::Created => { crc },
                        TaskStatus::Failed  => { crc },
                        _=> { panic!("module internal fsm state invalid (on create)"); }
                    }
                }
                false => is_created();
            }
        }
        TaskStatus::NeedsRemoval => {
            match modify_mode {
                true => {
                    let rrc = task.dispatch(context, visitor, connection, TaskRequest { 
                        request_type: TaskRequestType::Remove, 
                        changes: None 
                    };
                    match rrc.is {
                        TaskStatus::Removed => { rrc },
                        TaskStatus::Failed  => { rrc },
                        _=> { panic!("module internal fsm state invalid (on remove)"); }
                }, 
                false => is_removed(),
            }
        }
        TaskStatus::NeedsModification => {
            match modify_mode {
                true => {
                    let mrc = task.dispatch(context, visitor, connection, TaskRequest { 
                        request_type: TaskRequestType::Modify, 
                        changes: qrc.changes 
                    });
                    match mrc.is {
                        TaskStatus::Modified => { mrc },
                        TaskStatus::Failed  => { mrc },
                        _=> { panic!("module internal fsm state invalid (on modify)"); }
                    }
                }
                false => is_modified()
            }
        },
        _ => { panic!("module internal fsm state invalid (on query)");  
    }
    return result;

}

