use crate::data_structure::StateMachineFile;
use crate::state::StateMachine;
use color_eyre::eyre::{eyre, WrapErr};
use include_dir::{Dir, include_dir};
use tracing::info;

static STATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/states");

pub struct StateMachineBuilder {
    active_state_files: Vec<StateMachineFile>,
    loaded_state_files: Vec<StateMachineFile>,
}

impl StateMachineBuilder {
    pub fn new() -> Self {
        Self {
            active_state_files: Vec::new(),
            loaded_state_files: Vec::new(),
        }
    }

    pub fn load_state_file(&mut self, state_file: StateMachineFile) {
        info!("Loaded state file {}.", state_file.id);
        self.loaded_state_files.push(state_file);
    }

    pub fn load_builtin_state_files(&mut self) -> color_eyre::Result<()> {
        for file in STATES_DIR.files() {
            self.load_state_file(hcl::from_slice(file.contents()).wrap_err_with(|| format!("error while parsing file {:?}", file.path()))?);
        }
        Ok(())
    }

    pub fn is_state_file_active(&self, id: &str) -> bool {
        for f in self.active_state_files.iter() {
            if f.id == id {
                return true
            }
        }
        false
    }

    pub fn loaded_state_file_ids(&self) -> Vec<String> {
        self.loaded_state_files.iter().map(|v| v.id.clone()).collect()
    }

    pub fn active_all_state_files(&mut self) -> color_eyre::Result<()> {
        for i in self.loaded_state_file_ids() {
            self.activate_state_file(&i)?
        }
        Ok(())
    }

    pub fn activate_state_file(&mut self, id: &str) -> color_eyre::Result<()> {
        if self.is_state_file_active(id) {
            return Ok(());
        }

        for f in self.active_state_files.iter() {
            if f.id == id {
                return Ok(());
            }
        }

        let mut loaded = false;
        let mut deps = Vec::new();
        for f in self.loaded_state_files.iter() {
            if f.id == id {
                self.active_state_files.push(f.clone());
                info!("Activated state file {id}.");

                deps = f.depends.clone();
                loaded = true;
            }
        }
        if loaded {
            for d in deps {
                self.activate_state_file(&d)?;
            }
            Ok(())
        } else {
            Err(eyre!("unable to activate, state file {id} does not exist"))
        }
    }

    /// Sort the files using Kahn's algorithm in order to ensure we merge correctly.
    fn sort_state_files(&mut self) -> color_eyre::Result<()> {
        if self.active_state_files.is_empty() {
            return Ok(());
        }

        let mut destination = Vec::new();

        let mut s = Vec::new();

        // Step 1: Find initial s.
        for state_file in self.active_state_files.iter() {
            if state_file.depends.is_empty() {
                s.push(state_file.clone());
            }
        }

        if s.is_empty() {
            return Err(eyre!("no state files are without dependencies"));
        }

        while let Some(id) = s.pop() {
            destination.push(id);

            // Any state whose dependencies are already in the destination or s is viable.
            for state_file in self.active_state_files.iter() {
                if (!state_file.depends.iter().any(|i| {
                    !destination.iter().any(|f| &f.id == i)
                })) && (!destination.contains(state_file))
                    && (!s.contains(state_file))
                {
                    s.push(state_file.clone());
                }
            }
        }

        if destination.len() == self.active_state_files.len() {
            self.active_state_files = destination;
            Ok(())
        } else {
            Err(eyre!("failed to sort state files"))
        }
    }

    pub fn build(mut self) -> color_eyre::Result<StateMachine> {
        info!("Constructing final state machine...");
        self.sort_state_files()?;

        let mut sm = StateMachine::default();

        for f in self.active_state_files {
            info!("Merging state {}...", f.id);
            sm.merge_states(f.states);
        }

        info!("Performing sanity checks...");
        for state in sm.states() {
            let s = sm.get_state(&state).unwrap();
            for t in s.transitions {
                if sm.get_state(&t.target).is_none() {
                    return Err(eyre!("State {} does not exist", t.target));
                }
            }
        }

        //TODO: Do sanity checks to ensure the final SM is valid.

        info!("Done! Total states = {}", sm.states.len());
        Ok(sm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_all_states() -> color_eyre::Result<()> {
        let mut builder = StateMachineBuilder::new();
        builder.load_builtin_state_files()?;
        builder.active_all_state_files()?;
        let _sm = builder.build()?;
        Ok(())
    }

    #[test]
    fn state_ordering() -> color_eyre::Result<()> {
        let mut builder = StateMachineBuilder::new();
        builder.load_builtin_state_files()?;
        builder.active_all_state_files()?;
        builder.sort_state_files()?;
        for i in 0..builder.active_state_files.len() {
            let s = &builder.active_state_files[i];
            'd_check: for d in s.depends.iter() {
                for j in 0..i {
                    if builder.active_state_files[j].id == d.as_str() {
                        continue 'd_check;
                    }
                }
                assert!(false, "dependency {d} of state {} is not ordered before it", s.id);
            }
        }
        Ok(())
    }
}
