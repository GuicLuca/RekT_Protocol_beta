/**
 * LogSource are used to display prefix and filter log messages.
 */
pub enum LogSource {
    DatagramsHandler,
    PingSender,
    DataHandler,
    HeartbeatChecker,
    TopicHandler,
    ClientManager,
    ObjectHandler,
    Other,
}