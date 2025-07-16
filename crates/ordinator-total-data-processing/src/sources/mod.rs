pub mod baptiste_csv_reader;
pub mod baptiste_csv_reader_merges;

// This should be abstracted out be the. All this should be moved to the
// builder. You are

// This should be moved to the `scheduling-environment`
// ISSUE
// This is not a good thing to be doing. I think that you should pull this
// out of the code.
// TODO [ ]
// This should be handled be a specific component in the [`Orchestrator`]. I
// think that the best thing to be doing is creating the component, and then
// have a test and a prod setup. I think that the best approach now is to make
// the system work with the.
// ESSAY
// How should the time be handled here? The issue is that in Production the time
// would be handled continuously in the code, meaning that when a two week
// period runs out a new period is crated, and this should be uphold
// continuously to make.
//
// So what does that mean for this component? I think that it means that the
// code... The code should not simply work by constructing all the [`[Period]`]
// at once. The timer should be ticking and continuously create the new [`Day`]s
// and other types that are needed for this to work correctly. This is crucial.
// On a top level the orchestrator should be bound by a trait, which implements
// [`System`] clock. And here there should be at least two types. I think that
// each of these should then be created so that the system will act on the
// information produced by the clock. I think that it is also clear that the
// code and architecture should move towards a event-sourced model. I do not
// think that there is another way around that. The issue becomes where to draw
// the boundary
//
// QUESTION [ ] 2025-06-29
// How do we turn Ordinator into an Event sourced application?
//
// I think that the
// best approach here is to make the system work with events.
//
// Each message into the system would be an event, that updates the state of the
// Actor... No it would be its own event that is stored, this stored event would
// then be a used to construct the Actor itself.
//
// Events :
// * Each metaheuristics main loop iteration would then also be an event.
// * Tick of the [`SystemClock`]  would also be an event.
// * Changes to the [`SchedulingEnvironment`] would also be an event. This would
//   natually also implement delta evaluation for the LNS algorithms.
// * Event based systems would also remove the boundary of the time dependent
//   aspects of the system as you would simply have that every component of the
//   system acts on the [`SystemClock`] and this means that the whole system
//   naturally converges to what it should.
// Okay so I think that the best approach here is to make a trait for the
// [`SystemClock`] and then have the Orchestrator handle this. I think that the
// clock should send a messages for the [`Orchestrator`] every second. I think
// that much of the internal logic of the application should actually be in
// here. Meaning that when we start the system, we should make sure that the
// system initializes all the different periods until the required state
// is achieved. This will generalize the whole system to take
// care of how the system should handle time specific things. As
// we will naturally get rid of the "Start and initialize now" issue.
//
// QUESTION [ ] 2025-06-29
// * What should the plan be for fixing this? So this issue becomes now what we
//   should do to fix the issue with the implementing this. There are so many
//   different component that requires this setup to work properbly. You have 4
//   different types of events, I think that you should start with the correct
//   events here that is the best way of doing this. There is no need for a big
//   refactor here so you should not even start it? [ ] Adding the
//   [`SystemClock`] [ ] Have it send a message to the [`Orchestrator`] [ ] The
//   Orchestrator sends the message to the [`Actor`]s [ ] Make a
//   `TestSystemClock` and a `ProductionSystemClock`
//
// * Is there an iterative approach of doing this? Yes there is! Do not over do
//   this! Handle the issues when they occur. The big issue is the change of the
//   [`SchedulingEnvironment`] I think that we need to make the system work
//   correctly with events here. Meaning that data comes from the outside and
//   then the required parts of the [`SchedulingEnvironment`] changes, yes. And
//   initially this might be most of the [`WorkOrders`] but that is also okay.
//
//
// This is a huge refactoring of the whole system.
