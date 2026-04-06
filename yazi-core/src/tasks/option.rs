use yazi_scheduler::plugin::PluginInEntry;

#[derive(Clone, Debug)]
pub enum TaskOpt {
	Plugin(PluginInEntry),
}
