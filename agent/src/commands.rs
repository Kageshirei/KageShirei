use mod_agentcore::instance;

pub fn command_handler() {
    unsafe {
        if !instance().session.connected {
            return;
        }

        return;
    }
}
