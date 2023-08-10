use crate::playbooks::language::{AsInteger};

pub trait TaskProperties {
    
    fn get_when(&self) -> Option<String>;
    fn get_changed_when(&self) -> Option<String>;
    fn get_retry(&self) -> Option<AsInteger>;
    fn get_delay(&self) -> Option<AsInteger>;

    /** FIXME: add failed_when, other keywords ... **/
}

pub trait IsTask: TaskProperties { // + Runnable?
    fn run(&self) -> Result<(), String>;
}

#[macro_export]
macro_rules! define_task {
    ($name:ident { $($fname:ident : $ftype:ty),* }) => {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            pub name: Option<String>,
            pub when: Option<String>,
            pub changed_when: Option<String>,
            pub register: Option<String>,
            pub delay: Option<AsInteger>,
            pub retry: Option<AsInteger>,
            $(pub $fname : $ftype),*
        }
    };
}

#[macro_export]
macro_rules! add_task_properties { 
    ($T:ident) => {
        impl TaskProperties for $T {
            fn get_when(&self) -> Option<String> { return self.when.clone() } 
            fn get_changed_when(&self) -> Option<String> { return self.changed_when.clone() }
            fn get_retry(&self) -> Option<String> { return self.retry.clone() }
            fn get_delay(&self) -> Option<String> { return self.delay.clone() }
            fn get_retry(&self) -> Option<String> { return self.retry.clone() }
            fn get_register(&self) -> Option<String> { return self.register.clone() }

        }
    }
}