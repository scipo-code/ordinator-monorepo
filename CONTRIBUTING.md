# Requirements checklist
For every business goal:
> business goal -> user requirement -> system requirement -> test case

# Requirement Completeness
Scope: Does every Business Goal have a corresponding set of software requirements?
Logic: Have all CRUD operations and State Transitions (including errors) been defined?
Quality: Have constraints (Performance, Security, Compliance) been defined for every function?


## The "MECE" Stakeholder Scan
Completeness of requirements MECE principle (Mutually Exclusive, Collectively Exhaustive) to map your stakeholders.

Primary Users: The obvious daily users.
Secondary Users: Managers, admins, or reporting teams.
Tertiary/Silent Stakeholders: Regulatory bodies, security auditors, maintenance/DevOps teams, and support staff.
Negative Stakeholders: Those who might be negatively impacted by the system (e.g., a team whose manual work is being automated).


## Error handling
Error handling is used extensively to test the validity of the system. That means that
Results are used for business logic invariants as well. To make the system standardized
colors are used to standardize the understanding of the colors.

* Time related: Green
* Job related: Yellow
* Resources related: Blue
* Objective: Purple

Every error at runtime should ideally produce a single bug fix on every iteration. If this
is not upheld it means that the error was created in a wrong way.
