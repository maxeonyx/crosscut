# Catalogue: seed concerns

These are concerns from the common landscape that most software shares, written to the standard CrossCut aims for. They come from seven rounds of calibration with a human: most were rated "Perfect", and the rest were written to the same standard.

They lead by example. They are **not** a checklist:
- **A seed with real instances here is a good card.** Use its rule as written and bring the instances. A landscape padded with seeds that have no instance here has failed.
- **Never over-apply a seed.** Raise one only when this software gives it a real story and a concrete instance. A seed that does not fit is left out, never stretched to fit and never padded in as "not applicable".
- **Use them to learn the grain.** Each is one small rule, true as written, grounded in a concept a computer scientist would name, and backed by several concrete examples that include where it does not apply. Your discoveries should read like these.

## What made a rule Perfect, and what sank the rest

Max's own words from calibration, quoted:

- **It is a rule, small and literally true.** Title in the rule form ("X should Y"), hedged only where real exceptions exist: "Needs 'generally', otherwise can't be true as written. Sorry. That shit is important!!!!!" But "'should not' rather than 'should never' unless we're really confident".
- **Academic over layspeak.** "We should err towards academic language in the title rather than towards layspeak." Name the concept (monotonic clock, TOCTOU, birthday bound) and explain the jargon in the justification.
- **Grounded, not a bag of situations.** "Is there a more mathematical way to justify it or express it? Right now it's certainly agreeable but feels a bit ungrounded, like a 'you know it when you see it' bag of situations."
- **General, never about one project.** "It's supposed to be generalized!!!! We don't care about your demo projects!!!" No "the SDK", "the release", "the watchdog" in a rule.
- **Not on the default path.** "If things would be obviously broken if the concern was not managed then it's not CrossCut's job to bring it up." "Crosscut is helping us think broader, not think earlier about stuff we will encounter anyway."
- **Not a gotcha hunt.** "'Gotcha' in the 'trying to find a gotcha' bad way, not the 'real gotcha that's easy to forget about' good way." "If you can think of it → agents can think of it → another agentic tool can do it instead of us. Engineering taste not required."
- **Not change-shaped.** Migrations, API changes and renames: "A change is not a property of the codebase(s)."
- **Not compliance, not obligation.** "Is there a linter" is not a concern. Security framing counts against a concern.
- **Not too absolute, not overprescribing.** "It depends on the semantics of the job. So that makes it weak / incorrect. It's overprescribing. We need to err on the other side."
- **Not a lecture, not over-hyped.** "'Most builders have never thought of this.' Not sure that's true. Bad sentence, can be read as condescending."
- **Leverage beats importance.** The archetypal win is cheap and pays everywhere. A rule that needs a re-architecture to act on is weak: "Since this likely requires some kind of rearchitecture, it's not very actionable."

## Rejected, and why

Learn from these as much as from the cards:

- "Signing keys should not exist only on the machine that builds", ranked first for two sideloaded personal apps: "It's security flavored, first of all, which is a red flag. And it simply isn't actionable or valuable for the project."
- "Alarms set with the operating system should be set again after anything that can clear them": "Yes, but also we want to generalize it. So it's like state managed outside of the an app, you know. Could apply to Windows Registry. It could apply to whatever iOS does."
- "Scheduled actions should be checked as actually set well before they are due": "Then you call this dead man switch or positive confirmation. I feel like that would be a better title because it's more general. And then should say like examples for Android or whatever."
- "Shell scripts survive a Windows checkout": not a concern. "Not managing this causes shit to break in an obvious way, so it's on the default path."
- "Merge gates should depend on a job that always runs": weak. "The impact is 'temporary dev blocker' and it's just a stupid 'clever' bad-style gotcha."
- "A state change and the message announcing it should succeed or fail together": weak. "Badly generalized. Use computer science concepts... what's the practitioner supposed to do with this?"
- "An entity's status should change only through defined transitions": weak. "Too subjective... There's nothing fundamentally wrong with using free text to store state, frankly. It's just a choice."
- "Every retry loop should have a limit": "*Every* is a strong, strong bar to meet (do we have a mathematical proof or something resembling it?)".
- "Does every PR get two approvals?" and "Does every repository have a CODEOWNERS file?": compliance-shaped.
- "Is harbour-api's CLAUDE.md too long?": one project only.
- "Cyclomatic complexity under 10": "'is there a linter' is not a concern".
- "Migrations compatible with running code": "on agent and human default correctness pathway. It'll be solved on the happy path by any smart engineer."
- "Feed the watchdog where the work happens": "We can't have 'the' anything in the title. It's not a cross cutting concern in that case."

## The cards

### Elapsed time should be measured with a monotonic clock

A monotonic clock only moves forward, and setting the time of day doesn't change it. The wall clock is for timestamps: time synchronisation or a person can step it in either direction, so a duration computed from two wall-clock readings can come out wrong, huge or negative.

- A timeout computed from wall-clock readings fires at once when time synchronisation steps the clock forward.
- A benchmark reports a negative duration after a laptop syncs its clock.
- A virtual machine restored from a snapshot corrects its wall clock by several hours, and every retry delay computed from wall-clock time either fires at once or waits for hours.
- The wall-clock 'now' is the first time function people reach for. It is right for timestamps and wrong for durations, and most languages offer both: Date.now and performance.now in JavaScript, time.time and time.monotonic in Python, System.currentTimeMillis and System.nanoTime in Java.
- A monotonic reading means nothing outside the process or machine that took it, so a duration that spans machines or restarts still needs timestamps, with the care about clocks that comes with them.

### A new push should generally cancel its branch's in-progress CI runs

Superseded work: once a newer commit is pushed to a branch, an older run tests a version that has already been replaced, so finishing it spends runner time and delays the run whose result will be used.

- A developer pushes three fixups in five minutes, and the third run waits behind the first two for runners.
- A team's CI minutes double in a busy week, mostly on runs whose results nobody looks at.
- Concurrency groups are one line in the common CI systems (cancel-in-progress in GitHub Actions, interruptible jobs in GitLab), and they are off by default.
- It applies to pull-request and feature branches. On main and release branches every commit's result can matter, for bisecting or for deploys, and a deploy should never be cancelled halfway, so it is usually left off there.

### Deletions should propagate to every derived copy

A deletion can only travel if something carries it. Replicated and derived stores need either tombstones or delete events, or a sync that compares the whole set (full replacement or reconciliation), so that whatever is missing from the source is removed from the copy. Change-data-capture tools emit delete events; a sync that polls for recently changed rows can't see a deletion, because a deleted row has no newer version to find.

- A user deletes a post, and it keeps appearing in search results for a week, because the indexer only ever handled inserts and updates.
- An incremental sync copies rows whose updated_at is newer than its last run, so a hard-deleted row is never seen, and the warehouse keeps counting cancelled subscriptions forever.
- A nightly job that rebuilds the search index from scratch handles deletions for free; the incremental updates between rebuilds don't.
- A CDN keeps serving an image that was removed, because the delete path never purged it.
- Backups are the deliberate exception: they are snapshots meant to preserve the past, not derived copies that serve current reads.

### Concurrent cache misses for one key should share one underlying read

The cache stampede (or dogpile): when an entry is missing, every request that arrives before it is refilled goes to the source, so the moment a cache helps least is the moment the source is hit hardest. Request coalescing, leases and early probabilistic refresh are the established fixes.

- A popular key expires, and five thousand concurrent requests all miss and query the database in the same second.
- A deploy restarts every instance with an empty in-memory cache, and the database falls over on the cold start.
- A warm-up job gives every key the same one-hour lifetime, so an hour later they all expire together. Adding jitter to lifetimes spreads the refills out.
- In JavaScript, caching the in-flight promise instead of the resolved value makes concurrent callers share one read for free.
- Across a fleet, one read per node is usually enough. Go's singleflight, CDN request collapsing and memcache leases all exist for this; application caches built by hand rarely use them.

### A fallback that degrades the result should be observable whenever it is in use

Graceful degradation keeps a system useful when a dependency fails; silent degradation keeps it looking fine while it does something nobody chose. A fallback that is as good as the original, such as a second replica serving the same data, needs no signal. One that gives worse results is a signal like any other: for operators always, and for users when the difference matters to them.

- A recommendation service goes down and the page falls back to 'popular items'. It works well enough that nobody notices for three weeks, until someone asks why conversion dropped.
- A feature-flag SDK can't reach its server, silently serves its defaults, and turns a feature off for every user.
- Missing translations fall back to English, so users see a half-translated app and the translators never learn which strings are missing.
- A report job that can't read today's source republishes yesterday's data under today's date: the fallback hides the failure better than a crash would have.
- Routine fallbacks, such as serving a stale cached page while it refreshes, need only a metric; a fallback that changes what users see may deserve telling them.

### A tool running without a terminal should not wait for interactive input

A program can tell whether a person is at the other end: standard input is a terminal or it isn't (isatty). Without one, a prompt has no one to answer it, so the program waits until something kills it, or takes whatever happens to be on standard input as the answer. It should take the answer from a flag or the environment, or fail at once with a message naming the flag.

- A deploy script runs a CLI that asks 'Continue? [y/N]', and the CI job sits silent until its one-hour timeout.
- An agent runs the same command, sees no output, and its tool call hangs until it times out.
- Installing the tzdata package in a Docker build stops to ask for a time zone, which is why so many Dockerfiles set DEBIAN_FRONTEND=noninteractive.
- Tools that handle this well fail at once with a message such as 'no terminal: pass --yes to confirm', which turns a silent hang into an error that says what to do.
- Secrets are the exception to flags: they belong on standard input or in the environment, not on the command line.

### A scheduled job should be safe to run concurrently with itself, or concurrent runs should be prevented

Mutual exclusion: a run that takes longer than its schedule's interval overlaps the next one, and whether that is harmless depends on the job. Schedulers decide by default, and differently (cron always starts another run, systemd timers skip while one is active, Kubernetes CronJobs allow overlap unless told otherwise), so the job's own safety has to be known and matched to the setting.

- A billing job that normally takes ten minutes takes seventy at month end. The next run starts beside it, and both charge the same customers.
- A report job is installed on both servers of a failover pair, and it runs twice every night.
- A cache warm-up job would be safe to overlap, but the scheduler skips overlapping runs, and the cache stays cold for an hour.
- When runs are prevented, a run that hangs blocks every later one, so prevention needs a maximum run time too.

### Only plausibly transient failures should be retried

Transient and permanent faults: a retry can only succeed if the cause might have gone away (a timeout, a dropped connection, an overloaded server). Retrying a permanent one (a validation error, a missing permission, a declined card) repeats the failure, adds load and delays the real error. This relies on the failure saying what kind it is, which comes first, and on the operation being safe to repeat, which is a separate question.

- A payment client retries a 'card declined' response three times, and the customer's bank shows three declined attempts.
- A worker retries a message that fails schema validation, and every attempt fails the same way while the queue backs up behind it.
- A mobile app retries an upload the server rejected as too large, every few seconds, for days.
- Retry helpers are often wrapped around a whole call and retry every error alike, although almost every protocol already separates the kinds (HTTP 4xx against 5xx, gRPC status codes, database error classes).
- 'Plausibly' matters: a 503 is usually transient, and so can be a 404 for a record another service is still creating. The classification is a judgement about likelihood, not a certainty.

### Checking shared state and acting on the result should be one atomic step

Time-of-check to time-of-use (TOCTOU): between reading shared state and acting on what was read, another process can change it, so the action is taken on a fact that is no longer true. Making the check and the action atomic (a conditional write, a unique constraint, a transaction at the right isolation level, a lock) closes the gap.

- Two customers book the last seat within the same second: both requests check that it's free, and both write a booking.
- A sign-up form checks that a username is free, and a second request between the check and the insert creates a duplicate account; a unique index would have refused it.
- A program checks that a file doesn't exist before creating it, and another process creates it in between; opening it with 'create only if new' (O_EXCL) makes the check and the creation one step.
- A balance is read, compared with a withdrawal and written back, and two withdrawals at once both pass the check.
- Some checks only guide a person, such as 'this name is probably available'; the rule is for checks that an action depends on for correctness.

### Random assignments should be derived from a hash of what they assign, or drawn once and stored

Some random choices are used at once and forgotten (a retry's jitter, a shuffle for display). Others are assignments that later code relies on (which variant a user sees, which rows are in the test set), and those have to come out the same every time they are computed: derived from what they are about, as feature-flag systems do by hashing flag and user, or drawn once and stored. Drawing them again silently changes decisions that were already acted on.

- An A/B test assigns variants with a fresh random draw per session, so a user sees one variant on their phone and the other on their laptop, and the experiment measures a mixture.
- A machine-learning dataset's train/test split is redrawn whenever the dataset is rebuilt, so examples from the old test set leak into training and the reported accuracy is inflated.
- A job that picks a random sample of users for a survey redraws the sample on every run, so the reminders go to different people from the ones who were invited.
- A load balancer assigns sessions to servers at random and keeps the table in memory, so a restart moves every session to a new server.
- Consistent hashing and hash-based bucketing are the established tools; the failure is a plain random() call where an assignment was meant.

### A daily total's time zone should be made explicit wherever it could be ambiguous

A 'day' is a local calendar concept, not a fixed 24-hour span: its boundaries depend on a time zone, and some days have 23 or 25 hours. Every per-day, per-week or per-month figure has chosen a zone (UTC is a choice too). When everything a reader sees uses the one zone they already assume, that is enough; the zone needs stating when figures cross zones: a UTC computation for a local business, users in several zones, or two reports that chose differently.

- A New Zealand shop's 'revenue today' is computed with UTC day boundaries, so its day starts at lunchtime yesterday, and the dashboard never matches the till.
- The dashboard buckets by UTC and the finance export by local time, so every day has two different totals and month-end reconciliation takes a day.
- A global product computes daily active users in its servers' time zone, and after a move to another hosting region, its days no longer line up with the ones before.
- The day the clocks go forward has 23 hours, so its total looks like a dip every year.
- Weekly totals start on Sunday in one report and Monday in another (ISO 8601 weeks start on Monday); stating the zone and the week start makes the figures comparable.

### A missing or empty value should not widen the scope of a destructive operation

Empty values are identity elements. A filter with no conditions is true of every row (the empty conjunction is true), an empty path prefix is a prefix of every path, and a query builder that skips conditions for empty inputs turns an empty list of IDs into no condition at all. So when a destructive operation is built from values that can be missing, each missing one widens it, and the widest case is everything. A missing limit should be an error, or select nothing.

- An admin API deletes records using filters built from optional fields; a request without the tenant field builds a query with no tenant condition and affects every tenant.
- A bulk action deletes the selected items by ID, and when the selection is empty the query builder drops the ID condition, so the action applies to every item.
- A retention job deletes rows older than a configured age; when the setting is missing it reads as zero, and the job deletes everything older than now.
- An uninstall script built a path from a variable that came back empty, and the empty prefix matched the user's whole home directory (Steam for Linux, 2015).
- Operations meant to act on everything can still exist; they should ask for it explicitly (an --all flag, a separate 'delete all' action), so that everything is never the result of a missing value.

### A resource should not outlive its owner, even when the owner fails

Ownership and lifetimes: RAII and structured concurrency tie a resource to the scope that owns it, and leases do the same across machines, since a holder that dies stops renewing and loses the resource. The principle runs from memory and threads to processes, cloud resources, access rights and records in a domain model. It is hard to achieve in full, because failures skip clean-up code; the reliable forms make the lifetime structural (a scope, a lease, an expiry, a cascade) rather than a step that runs only on the success path.

- A goroutine started for a request keeps running after the request is cancelled, holding a database connection, because nothing ties its lifetime to the request's.
- A test runner is killed, and the containers it started keep running on the developer's laptop.
- A CI job creates a cloud database for each pull request and deletes it only when the job succeeds, so every failed run leaves one behind.
- A contractor's project ends and their access to production stays, because the access was granted to the person rather than to their role on the project.
- A customer is deleted, and their scheduled emails keep sending, because the schedule has no owner once the customer is gone; a cascade, or a foreign key that forbids the orphan, would have ended it.

### A service asked to stop should stop taking new work, then finish or cleanly cancel what it has

Graceful shutdown has two halves. Admission: stop accepting new requests and messages, so load balancers and queues route around the process. Completion: finish in-flight work within the grace period, or cancel it cleanly, rolling back or handing messages back to the queue, so nothing is left half done. Orchestrators stop processes routinely (every deploy, every scale-down), so this runs far more often than a crash.

- Every deploy of an API drops a handful of requests, because the process exits the moment it is asked to stop while the load balancer is still sending it traffic.
- A queue worker stopped halfway through a message leaves the work half done, because it neither finished nor handed the message back.
- A service handles all of this correctly and it never runs: in its container, a shell is PID 1 and doesn't pass the stop signal on, so the service is killed at the end of the grace period without ever being asked to stop.
- Development is the cheapest place to build this: handling Ctrl-C (SIGINT) and an orchestrator's stop signal (SIGTERM) the same way gets most of the way there, and every Ctrl-C then exercises the shutdown path.

### Derived data that can go stale should carry its as-of time

Derived data (a report, a dashboard, a search index, a materialised view) lags its source, and a refresh that fails leaves it looking exactly like current data; its as-of time is the only visible difference. With several sources it is harder: a figure built from sources refreshed at different times is only as current as its oldest input, and inputs from different moments can combine into figures that were never true together. The honest as-of time is the oldest input's, or one per source where they differ.

- A dashboard looks normal for three days while its nightly load has been failing.
- A weekly report is forwarded to management with last week's figures, and nothing on it says so.
- A search index falls an hour behind, and users conclude that a new document was never saved.
- A revenue dashboard joins orders refreshed hourly with refunds refreshed nightly, so each afternoon it shows today's orders against yesterday's refunds, and a single 'updated at' time would be wrong about half of it.
- A figure computed fresh on every request has nothing to show; this is for anything built from a copy.

### Each error should be logged once, where it is handled

Handling an error means deciding what happens next: retry, fall back, report it to the user, or give up. Only that decision knows how serious the error is (a missing record is routine to one caller and a bug to another), so only the handler can log it at the right level. Logging it at every layer it passes through, the log-and-throw anti-pattern, turns one failure into several error lines at severities that disagree, which inflates error rates and alerts and hides how many failures there really were. Layers in between add context by wrapping the error instead, so the one line carries the whole story.

- A database timeout is logged by the repository, by the service that called it, by the request handler and by the framework: one failure becomes four error lines with four different messages, and an alert set at 20 errors a minute fires at five real failures.
- A library logs 'ERROR record not found' before returning the not-found result, so an application that treats a missing record as normal fills its logs with errors it has already handled.
- A web front end reports an exception to its error tracker in a low-level fetch wrapper and again in the screen that shows the failure, so one bug appears as two issues with two different stack traces.
- A command-line tool prints an error where a file read fails and again at the top level before it exits, so users see one failure twice, in different words.
- Logging on the spot feels safe, because the code can't see whether anything above it will log the error; the convention that only the handler logs is what makes the single line trustworthy. Debug-level traces are a separate stream.

### Machine-readable text should be formatted and parsed the same way in every locale

Locale-sensitive functions format and parse for people: the decimal separator, digit grouping, date order and even letter case change with the user's locale. When they are used for text that programs read (a CSV export, a config file, a protocol message, a cache key), the same code writes different bytes on different machines, and parsing breaks for some users only. Most platforms provide an invariant form for this (C#'s CultureInfo.InvariantCulture, Java's Locale.ROOT, C's "C" locale), and linters can flag the calls that need it.

- A Java program lowercases the key 'USER_ID' with the default locale, and on machines set to Turkish it becomes 'user_ıd', with a dotless i, so the lookup fails there and nowhere else.
- A .NET app writes prices to a CSV with the current culture, and a user in Germany gets '1,50' in a comma-separated file, splitting every price across two columns.
- A C program calls setlocale for its user interface and starts writing '0,5' into the JSON it sends, which the server rejects.
- Developers' machines mostly share one locale, so the difference first appears on a customer's machine; running the tests once under a Turkish or German locale finds most of these.
- Text for people should do the opposite and follow the user's locale; the concern is keeping the two apart.

### Latency measurements should include requests that failed or timed out

Survivorship bias, or censored data: a request that times out took at least the timeout, and one that fails fast has a real latency too. Recording latency only on the success path drops the slowest and most broken requests, so during an outage the latency graph can look better than normal. Recording every request, labelled by its outcome, keeps both the distribution and the failures honest.

- A service records latency only after a successful response, so during a database outage, when most requests time out after thirty seconds, its p99 latency graph stays flat at 200 milliseconds.
- A latency objective counts slow requests as a share of successful ones, so requests that time out drop out of both sides, and the objective looks met during the worst hour of the year.
- A load test reports latency from the responses it received, and the requests the server dropped don't appear in the results at all.
- A front end measures page loads with an analytics beacon sent when the page finishes loading, so users who gave up before it loaded never appear in the data.
- Labelling each measurement with its outcome (success, error, timeout) keeps the good requests' distribution readable while still counting the rest; a timed-out request is recorded at the timeout.

### Random identifiers should be long enough that collisions stay improbable at the volume they'll reach

The birthday bound: among n values drawn at random from N possible ones, a repeat becomes likely once n nears the square root of N, not N itself. Six random digits give a million codes, but a repeat is more likely than not after about 1,200 of them; a random 32-bit ID reaches even odds at about 77,000. So a random identifier's length should be chosen from the square root of its space, or collisions should be checked for and retried.

- A shop gives each order a random six-digit reference, and two customers soon get the same one, and support can't tell their orders apart.
- A link shortener uses five random characters from 62, about 916 million codes, and a collision becomes more likely than not at about 36,000 links, long before anyone expected to worry.
- A file-upload service names stored files with a random 32-bit number and overwrites an earlier upload when two names coincide.
- Random version-4 UUIDs (122 random bits) need about 2.7 quintillion to reach even odds, which is why they can be used without a check; short codes meant for people are where the check matters.
- Where a code must be short, a uniqueness constraint and a retry on conflict make it safe; the failure is a short random code with neither.

### Metric labels should take values from a small, bounded set

A metrics system stores one time series per combination of label values, so the number of series is the product of each label's number of values. A label with unbounded values (a user ID, a URL with IDs in it, an error message) multiplies that product by every value it ever takes, and memory, query time and cost grow with it. Unbounded values belong in logs and traces, which store events rather than series.

- A developer adds a user_id label to a request counter, and the metrics server runs out of memory the next day.
- Request latency is labelled by URL path, so every /orders/48213 becomes its own series; labelling by the route template (/orders/:id) keeps it to one.
- A counter labelled with the error message grows a new series for every distinct message, including the ones with timestamps in them.
- Hosted monitoring services bill by the number of custom metrics or series, so one label can turn a small bill into a large one.
- Prometheus's own documentation warns against labels with unbounded values; the trap is that a new label looks harmless on a test system with ten users.

### An export should round-trip through its matching import without loss

The round-trip property: when a system can both export and import a format, importing an export should give back exactly what was exported; that is, import after export is the identity function. Each field the exporter leaves out, or writes in a form the importer can't read back exactly, is lost the first time the data moves, and the loss usually goes unnoticed until the data is needed. Deliberate exclusions, such as secrets or identifiers regenerated on import, aren't loss, provided they are stated.

- A customer moves to a new account with export and import, and every note, attachment and custom field is missing, because the CSV export only ever included the columns shown on screen.
- A dashboard is exported and imported into another instance, and every panel is broken, because the export refers to data sources by an ID that only exists in the old instance.
- A hand-written backup script copies every table's rows but not the sequences' current values, so after a restore, new inserts fail on duplicate keys until someone resets the sequences.
- A project file saved by version 3 and opened by version 3 loses the colour of every layer, because the saver writes a field the loader never reads.
- Property-based testing states exactly this as a law (decode after encode gives back the input), and a test that imports an export and compares the result finds many of these gaps.

### A cache should not store a failure as if it were a success

A cached entry stands in for what the source would return. 'The search found nothing' is a success; 'the search couldn't run' is a failure. Result types (Rust's Result, Haskell's Either) keep the two apart. When a failure is turned into an ordinary value (an empty list, a default, a null) before it reaches a cache, the cache serves it for as long as the entry lives, even after the source has recovered, and it looks exactly like a real answer. Caching a failure deliberately, as a failure and briefly, is a different and legitimate choice.

- A function catches a timeout from the pricing service and returns an empty list; the memoisation around it caches the empty list, and the product page shows 'no prices' for an hour after the service is back.
- A search-suggestions endpoint returns an empty list while its index is rebuilding, with its usual day-long cache lifetime, and browsers show no suggestions until the next day.
- A configuration client can't reach its server at start-up, falls back to defaults, and marks them as fetched, so it never tries again and the service runs on defaults until it is restarted.
- Caching a genuine 'not found' is fine, and so is caching an error briefly, as an error, to protect a struggling source; the failure is caching 'couldn't find out' as if it were 'found nothing'.
- The protocols draw the same line. HTTP caches may keep a 404 without being told to, but not a 503, and DNS lets resolvers cache a name that doesn't exist for as long as its zone says, but a server failure for at most five minutes (RFC 2308).

### An update should not erase fields the client doesn't know about

Forward compatibility in writes: a client built against an older schema reads a record, changes one field, and writes the whole record back; every field added since it was built is missing from what it sends, so a full-replacement update erases them. Partial updates (only the fields named change, as in JSON Merge Patch), or preserving unknown fields when round-tripping (as Protocol Buffers do), keep the newer data.

- A new 'delivery notes' field is added to orders; customers still on last year's mobile app edit their address, the app sends the whole order as it knows it, and the delivery note on every order they edit is wiped.
- An integration syncs contacts by reading them, applying its changes, and writing them back with a full replace, and the custom fields another team added disappear on every sync.
- Kubernetes' server-side apply records which client set each field, so a client's apply removes only the fields that same client set before, not ones added by other tools.
- Protocol Buffers originally dropped unknown fields in proto3 and later went back to preserving them, so that messages passing through older code keep their newer fields.
- Full replacement is fine when every writer knows the whole schema, or when one writer owns the whole record, as a configuration tool does with the files it manages; the risk grows with every client version still in the field.

### Identifiers should be compared in one canonical form everywhere

Canonical forms: an identifier's equality rules, such as whether case, Unicode composition or surrounding whitespace matter, define which strings count as the same identifier (an equivalence relation), and a canonical form picks one representative from each group of equal strings. Every layer (the form, the API, the database's unique index, the search, the file system) has to apply the same rules, or the same thing is stored twice and lookups miss. Choose the rules once, derive the canonical form from them, and compare and enforce uniqueness only on that form; the original spelling can still be kept for display. Unicode defines the normalisation forms (UAX #15) and case folding for this.

- A sign-up form lowercases email addresses, but accounts created through the API don't, and the unique index is case-sensitive, so Bob@example.com and bob@example.com become two accounts, and a password reset applies to the wrong one. The email standard technically allows case-sensitive local parts; almost no provider uses that, and which rule a system follows is a decision to make once, not per layer.
- A file whose name contains an accented letter is created on a Mac with the older HFS+ file system, which stores names in decomposed form; on Linux, the composed spelling of the same name is a different file, so a repository appears to contain two files with one visible name. Git's core.precomposeunicode setting exists because of this.
- The tag 'café' typed on a phone and pasted from a PDF can differ in their bytes, so the tag list shows it twice and filtering by one misses half the posts.
- A username with a trailing space is accepted at sign-up, and the person can never log in, because the login form trims it.
- People have standardised this in places: PRECIS (RFC 8264, with the username and password profiles in RFC 8265) defines how to prepare them for comparison, and GitHub treats usernames case-insensitively. Identifiers that are case-sensitive by design, such as tokens, have a canonical form equal to themselves.

### A write should not be acknowledged as more durable than it is

Durability and acknowledgement semantics: a caller that gets 'success' stops keeping its own copy and stops retrying. So an acknowledgement sent while the change could still be lost (in a buffer, an unsynced file, an unreplicated log, a request still in flight) means a crash at the wrong moment loses work that was reported as safe. Weaker guarantees are legitimate trade-offs, such as asynchronous commit or leader-only acknowledgement, as long as callers know how much the acknowledgement means.

- A web app shows 'Saved' the moment the user clicks, before the request completes; the request fails on a flaky connection, and the edit is lost with no error shown.
- An API returns 200 OK after putting a write in an in-memory queue, a deploy restarts the process before the queue is flushed, and the write is lost.
- A service publishes an event with leader-only acknowledgement and reports success to its own callers; the broker's leader fails a second later, before any replica has the event, and it is gone.
- A text editor reports a file as saved after writing a new copy and renaming it over the old one without syncing it; after a power cut, the file can be empty. Many applications hit this on ext4 in 2009. Syncing the file before the rename, and its directory after, is the durable pattern.
- Asynchronous work can be acknowledged honestly: HTTP's 202 Accepted means less than 200 does, and a database with asynchronous commit is fine when callers know a crash can lose the last second or so of commits.

### A timed-out request should be treated as having an unknown outcome

The two generals problem: when a request times out, the caller can't tell whether it never arrived, arrived and failed, or succeeded with the reply lost. Treating a timeout as a failure is a guess, and a wrong guess shows users 'failed' for things that happened, or makes code undo or repeat work that was already done. The outcome has to be found out (by checking, or by an idempotent retry), not assumed.

- A payment request times out, the checkout shows 'Payment failed, please try again', and the customer's card was in fact charged; the second attempt charges it again.
- A deploy tool times out waiting for a rollout and reports failure, so an engineer rolls back a release that had succeeded.
- A booking API times out, the client marks the booking as not made and releases the seat locally, while the server holds it for the customer.
- An agent's tool call times out, the agent assumes nothing happened, and it creates the same issue a second time.
- Idempotency keys make a retry safe; this rule is the step before that: not reading a timeout as a failure.
- The SQL standard names this state: error code 08007 means 'transaction resolution unknown', for a connection lost while a commit was in progress.

### Text that passes one layer's length check should fit every layer after it

Unicode text has several lengths: bytes in UTF-8, code units in UTF-16 (what JavaScript's length counts), code points (what many databases count), and user-perceived characters, called grapheme clusters (UAX #29). A family emoji made of three people is 1 character, 5 code points, 8 UTF-16 units and 18 bytes. When each layer counts differently, text accepted by one is rejected or cut by the next. Counting in one unit everywhere is the simplest fix; otherwise each earlier limit has to guarantee the later ones.

- A form allows 50 characters, counted by JavaScript, and the API allows 50 bytes; a name in Japanese passes the form and is rejected by the server with an error the user can't act on.
- A validator in JavaScript, one in Python and one in Go count the same text in UTF-16 units, code points and bytes respectively, and each is correct by its own language's definition, which is why no single layer looks wrong.
- A server truncates descriptions to 255 bytes and cuts through the middle of a multi-byte character, leaving invalid UTF-8 that a downstream parser rejects.
- File names on common Linux file systems are limited to 255 bytes, not characters, so names in scripts that need three bytes per character fail at a third of the length English names reach.
- SMS limits depend on the character set (160 characters in the basic GSM alphabet, 70 once any character outside it appears), so one emoji can split a message in two.

### A recurring job that can't overlap itself should have a maximum run time

Mutual exclusion guarantees safety (no two runs at once), but it only guarantees progress (the next run eventually happens) if each run holds the exclusion for a bounded time. When the next run skips or waits while the previous one is still going, one run that hangs, rather than crashing, holds back every run after it, silently and indefinitely. A maximum run time turns the hang into a visible failure and lets the next run go ahead.

- A nightly export hangs on a network call with no timeout. The scheduler skips a run while the previous one is still going, so every night after that is skipped too, without an error, and the missing exports are noticed three weeks later.
- A systemd timer doesn't start its service again while the previous run is still active, and a oneshot service, the usual kind behind a timer, has no start timeout by default. A run stuck waiting on a database lock holds the job back until someone finds it.
- A cron entry wrapped in flock to prevent overlap waits behind a stuck run, and so does every run after it, because the lock is held by a process that is alive but doing nothing.
- A desktop app syncs every five minutes and skips a sync while the previous one is running. One request that never returns leaves the sync marked as running, and the app quietly stops syncing until it is restarted.
- A crash is different: a crashed run ends, and the next one starts on time. Some platforms bound runs by default (GitHub Actions ends a job after six hours unless told otherwise); Kubernetes Jobs have a setting for it, activeDeadlineSeconds, which is unset unless someone sets it.
- The maximum should be generous, well above the longest normal run: it exists to end hangs, not to cut short a run that is merely slow.

### A program should start even when a service it can work without is unreachable

Failure isolation by criticality: a program that can do useful work without a service (analytics, a cache, a flag service, remote configuration, an update check) but refuses to start without it turns that service's outage into its own. Under an orchestrator it gets worse: every restart and every deploy during the outage fails, so an outage of something minor can take the whole service down. Failing fast at start-up is right for services the program truly can't work without, and wrong for the rest.

- An analytics SDK throws at start-up when it can't reach its endpoint; during the provider's outage, every pod that restarts crash-loops, and checkout loses capacity with each one.
- An app refuses to boot when its Redis cache is unreachable, though every request could have gone to the database, slower.
- A service can't start while its feature-flag server is down, although the flags have safe defaults it could have used.
- A mobile app waits on its splash screen for remote configuration, so with no signal it never opens, though it could have used the configuration from last time.
- A command-line tool checks for updates before doing anything, and on a flaky train connection it waits a minute before starting the job it was asked to do.
- Which dependencies are optional is a decision; the failure is never having made it, so that every dependency is effectively critical at start-up. This is separate from health checks: a service whose liveness check ignores dependencies still crash-loops if its start-up code exits.

### A user action that takes longer than about a second should be visibly in progress

HCI response-time limits (Card, Robertson and Mackinlay, 1991; Nielsen, 1993): about 0.1 seconds feels instant, about 1 second keeps a person's flow of thought, and beyond that they need to see that the system is working, or they may assume it isn't. The assumption is what causes the harm: people click again, reload, or leave.

- A 'Pay' button gives no sign of anything happening for three seconds on a slow connection, the customer clicks again, and places a second order.
- An export takes twenty seconds in production and a fraction of a second on a developer's machine, so the people building it never saw the blank wait users see.
- A form submits with no feedback, the user reloads the page to 'try again', and loses everything they typed.
- A command-line tool prints nothing for a minute while it downloads, and users kill it thinking it has hung.
- Indicators should follow the same limits: a spinner that flashes for 50 milliseconds on a fast response is noise, which is why many designs delay showing one.

### Every dataset should be either a source of truth or rebuildable from one

Systems of record versus derived data: a copy computed from sources can always be checked and rebuilt, but a copy that is neither a source nor rebuildable is a second source of truth nobody declared, and when it diverges, nobody can say which is right.

- A search index is corrupted, and nobody can rebuild it, because the code that fills it only ever handled incremental updates.
- A denormalised totals table drifts from the orders it summarises, and nobody can say which of the two is correct.
- An analytics table was built months ago by hand-run SQL, and now it feeds a board report that nobody can regenerate.
- A cache of a third-party API has quietly become the only copy after the API stopped serving old records, and it is still being treated as disposable.
- A cache holds webhook idempotency keys and nothing else does. When the cache restarts, retried webhooks are processed twice: the 'cache' was a source of truth all along.
- Derived stores usually begin as optimisations fed incrementally, so the full rebuild path is never written. Event-sourced systems make rebuilding a first-class operation; most systems only discover they need it during an incident.

### Identifiers should not encode facts that can change

Identity versus attributes (natural versus surrogate keys): an identifier has to stay constant while everything about the thing changes, and one that encodes a changeable fact must change when the fact does. 'Cool URIs don't change' (Berners-Lee, 1998) is the same principle for the web.

- An email address is a user's primary key. When someone changes theirs, dozens of tables need updating and every external system that stored it breaks.
- A document's URL is built from its title, and fixing a typo in the title breaks every link anyone ever shared.
- A product SKU encodes its category, so moving a product to a new category means renumbering it and reconciling every order that mentions the old number.
- A sync tool uses file paths as identity, so renaming a folder looks like deleting everything in it and creating it again.
- Encoding an immutable fact is fine: ULIDs and UUIDv7 carry their creation time, which never changes. GitHub's redirects for renamed repositories exist because usernames and repository names are changeable facts inside URLs.

### Message consumers should rely only on the ordering their channel guarantees

Ordering guarantees differ: none, FIFO, per key or partition, causal, or total. Retries and parallel consumers break order even on channels that promise it, so a consumer either handles messages in any order or uses sequence numbers to detect the ones that arrive late.

- A consumer treats a Kafka topic as globally ordered, but order only holds within a partition, and updates for different partitions interleave unpredictably.
- A webhook handler assumes events arrive in the order they happened; payment providers such as Stripe document that they may not, and 'paid' arrives after 'refunded'.
- A message fails, is redelivered a minute later, and is applied after three newer messages about the same record.
- Three parallel workers take messages in order from a FIFO queue and finish them in a different order.
- FIFO queues with message groups and per-key partitioning exist precisely to give the guarantee where it's needed; the failure is relying on one that was never there.

### A batch operation should either be atomic or report which items failed

Partial failure: a batch that can succeed for some items and fail for others has two honest shapes, all-or-nothing, or best effort with a result for each item (as in HTTP's 207 Multi-Status or per-item statuses in batch APIs). Reporting best effort as plain success hides the failures from everyone downstream.

- A bulk import of 10,000 rows returns 200 OK; 37 rows failed validation and were dropped, and nobody finds out until a customer asks where their data went.
- A CLI that processes a folder of files exits 0 after failing on three of them, so the script that called it carries on as if everything worked.
- S3's batch delete returns per-key errors inside a successful response, and a cleanup job that only checks the status code believes everything was deleted.
- A GraphQL response carries an errors array next to partial data, and the UI renders the partial data as if it were complete.
- Formats for per-item results exist almost everywhere; the gap is servers that don't fill them in, and clients that don't read them.

### Inputs should be parsed into validated types at the boundary

'Parse, don't validate' (Alexis King, 2019): checking input and passing on the raw value throws away the knowledge that it was checked, so every later function either re-checks or assumes. Parsing it once into a type (or schema-backed model) that can only hold valid values carries that proof through the whole program.

- A handler checks that a request body looks valid, then passes the raw dictionary on; three layers down, a function crashes on a field that one code path forgot to check.
- An environment variable set to 'false' is a non-empty string, so it is truthy, and a feature switches on in production.
- An email address is validated in the sign-up form, but the admin API accepts whatever it's given, and bad addresses reach the mailing system.
- One client sends an ID as a number and another as a string, and the two fail to match in a lookup far from where either arrived.
- In dynamically typed code, the 'type' is a schema-backed model such as Pydantic or Zod; the rule is about parsing once, where data enters, not about any particular type system.

### Paginated results should be sorted into a total order

Pages are only consistent if every pair of rows has a defined order, so ties in the sort key must be broken by something unique, such as (created_at, id). SQL guarantees no order at all without ORDER BY. Paging by key (after this cursor) rather than by offset also keeps pages stable while rows are being inserted.

- An API sorts by created_at; a bulk import gave 500 rows the same timestamp, so page 2 repeats some rows from page 1 and never shows others.
- A query paginates with no ORDER BY, and the database returns rows in a different order on different calls, so pages overlap unpredictably.
- A client syncs a list page by page using offsets while new rows are being added at the top, and every insert shifts the pages, so it skips some rows and fetches others twice.
- Sorting by name, or by any column with duplicates, has the same problem; adding the primary key as a final tie-breaker fixes it.
- It works perfectly in tests with a handful of rows that all have different timestamps, which is why it survives until real data arrives in bulk.

### A queue's backlog should be measured by the age of its oldest item

Little's law: the time an item waits equals queue length divided by throughput, so a count alone says nothing about delay. The age of the oldest item is the delay itself, and it is the only measure that catches a queue that is short but stuck.

- A queue with 10,000 messages looks alarming on the dashboard, but it drains in a minute and nobody is affected.
- A queue with 50 messages looks healthy, but its oldest message is six hours old, because the one consumer that handles it is stuck.
- A background-job dashboard shows only counts, so a job wedged at the head of a first-in-first-out queue holds up everything behind it for a day.
- Consumer lag reported as a number of offsets is meaningless without the write rate; reported as time, it says directly how stale the downstream data is.
- Some platforms expose it (SQS has an age-of-oldest-message metric); most job dashboards default to counts.

### Files that outlive their code should generally record their format version

Schema evolution: data outlives the code that wrote it, and a version tag lets later code migrate the data instead of guessing its layout.

- A game patch changes its save layout, and existing saves load as garbage because nothing in the file says which layout it is. Version 1 has nothing to differ from, so the field feels pointless at exactly the moment it is cheapest to add.
- A desktop app's settings file is read by a newer build that silently drops a field it no longer expects, and the user's preferences vanish.
- A design tool opens a project exported three years ago and has to guess, from the shape of the JSON, which of four historical layouts it is.
- It doesn't apply to caches rebuilt at start-up, or to formats such as Protobuf and Avro that evolve through compatibility rules instead of a version number.

### A stale copy should never overwrite a fresher one

Replicas and the session guarantees (monotonic reads, read-your-writes): once a newer version exists anywhere a reader or writer can see it, nothing should replace it with an older one. Versions make 'fresher' checkable. A deliberate rollback is a new write, not a stale copy.

- A page redraws an open text box from its state, which is half a second behind what the user just typed, and the typing disappears. This was the calibration page's own bug.
- A live sync delivers the server's copy of a note that is older than the user's unsaved edit, and the edit is replaced.
- A user renames a project, the next page load reads from a replica a second behind, and the old name comes back, so they rename it again.
- Two admins open the same settings form. The second saves last, from a copy loaded before the first admin's change, and the first change is silently lost. Databases have offered optimistic locking for this for decades; web forms rarely use it.
- A cache is refilled by a slow read that began before an update and finishes after the invalidation, writing the old value back, where it stays until it expires.
- Two browser tabs hold the same document. The older tab autosaves last and quietly undoes the newer one.
- A mobile app comes back online and uploads its offline copy of a record over changes made on the web in the meantime.
- An event consumer receives 'shipped' and then a delayed 'paid', and moves the order backwards.
- A slow deploy pipeline run finishes after a newer one and applies last week's configuration over this morning's hotfix.
- Someone runs an infrastructure apply from a checkout they haven't pulled in a week, and reverts everything changed since.
- A developer force-pushes a branch they fetched yesterday and erases a teammate's commits. Git's --force-with-lease exists for exactly this, as do HTTP's If-Match and Kubernetes' resourceVersion: the idea is well known locally, and rarely applied to every copy a system has.
- It needs a notion of 'fresher': a version number, a sequence, or a timestamp from one clock. When two writes are genuinely concurrent, neither is fresher; versions detect the conflict, and resolving it is a separate problem.

### Builds should be hermetic, depending only on inputs they declare

Hermeticity: when a build's output depends only on declared inputs, a difference in output always traces to a declared change, which is what makes bisection, caching and 'works on my machine' tractable. Bazel and Nix enforce it; most builds merely hope for it.

- A release job installs the compiler through the system package manager at build time, so the released binary is built by a newer compiler than any developer has, and a bug appears only in the release.
- A build downloads a helper script from a URL during every run. Months later the URL moves, and a checkout of unchanged source no longer builds.
- A project builds on one developer's laptop only, because it reads a registry token and a mirror setting from their home directory. Agents working in clean sandboxes can't build it at all.
- Hermeticity is a direction, not a switch. Declaring the toolchain version, taking dependencies only from the lockfile, and removing network fetches from the build gets most of the benefit without a fully sealed toolchain. Byte-identical output (determinism) is a related but separate property; it matters most for caches.

### Importing a module should generally have no side effects

Definition versus execution: loading code should only define things. When it acts, import order becomes hidden semantics, loading becomes slow and environment-dependent, and every tool that merely inspects code (test collectors, type checkers, doc builders, agents) sets off real work.

- A test imports a configuration module that opens a database connection as it loads, and the test hangs on any machine without that database.
- A CLI takes three seconds to print --help, because importing it initialises an SDK and reads credentials from disk.
- Two modules each configure logging when imported, so whichever happens to load first decides the log format for the whole program.
- A test runner collecting tests imports every module, and one of them sends a real request to a partner API each time anyone runs the suite.
- Registering definitions (route decorators, plugin registries) is the accepted exception: it records what exists without doing anything. Serverless platforms recommend building clients at module scope for warm reuse, and that fits too, as long as the connection opens on first use.

### A record of progress should advance only after the work it records is durable

Commit ordering, as in write-ahead logging and at-least-once processing. A watermark, a processed-ID list or a consumer offset is a claim that some work is done. If it is written before the work's effects are durable (uploaded, committed, flushed), a crash in between loses that work permanently, because the next run believes it already happened. If it is written after, a crash only causes a repeat, which is safe when the work is idempotent. So the progress record goes last, and the work has to be safe to repeat.

- A pipeline records a notice as processed, then the upload of its attachment fails; the next run skips it, and the attachment never appears.
- An incremental ETL job advances its high-water mark at the start of the run, the load fails halfway, and the rows in that window are never loaded.
- A Kafka consumer commits its offset on receipt rather than after handling the message, and a crash drops the message.
- A job writes its state file and its outputs to different stores, and only one of the two writes survives a timeout.
- Where losing an item is acceptable, as with best-effort metrics, the ordering matters less.

### Published output should become visible all at once, or be marked provisional or complete

Atomic visibility, as in commit protocols. A consumer that reads while a producer is writing sees a state that never existed: half a file, a new index page that links to pages not yet uploaded, some extract files without their partners. The established patterns are to write to a temporary name and rename, to write the data first and a manifest or completion marker last, or to swap a pointer to a fully built version.

- A downstream system picks up an extract file while it is still being written and loads half of it.
- A static site uploads its new index page before the attachment it links to, and readers get a 404 for a few minutes, or forever if the upload then fails.
- A consumer polls a directory for a CSV and its matching control file, and processes the CSV alone because the control file arrives second.
- A dbt run rebuilds a table in place, and a report queries it mid-run.
- A single small object written with one atomic PUT to an object store is already atomic.

### A stream consumer should be able to detect gaps in what it received, and recover from them

Sequenced delivery. Over a connection that can drop, a consumer cannot tell 'nothing happened' from 'I missed it' unless messages carry sequence numbers or the protocol supports resumption. Detecting a gap is the precondition for recovering from it, by replaying from a sequence or re-fetching a snapshot. Server-Sent Events has this built in (event IDs, and the Last-Event-ID header on reconnect), but only if the server assigns IDs and can honour them.

- A browser's market ticker reconnects after a network blip and silently misses three price updates, showing a stale price until the next trade.
- A service consuming an exchange feed loses a UDP packet, and its order book is wrong until restart, because nothing checked the sequence numbers.
- A chat client reconnects and never shows the messages sent while it was offline.
- A stream of 'current value' updates that each carry the full state needs only to resubscribe, not to replay.

### A snapshot followed by a stream of updates should not lose the updates that happen in between

This is the snapshot-and-delta consistency problem. If a consumer takes a snapshot and then subscribes, any update between the two is lost. If it subscribes and then takes a snapshot, updates already reflected in the snapshot may be applied twice. The standard fix is to subscribe first and buffer, take a snapshot that carries a version or sequence, and then apply only buffered updates newer than it. Change-data-capture initial loads and market-data recovery protocols both specify exactly this.

- An SSE endpoint sends the current prices and then attaches the client to the live stream; a trade in the gap never reaches that client.
- A warehouse does a full initial load from a source table and then starts consuming changes from the time the load finished, missing the rows changed during the load.
- A cache warms itself from a database scan and then subscribes to invalidations, keeping stale values for keys updated mid-scan.
- If updates carry full state and arrive frequently, a missed one self-heals at the next update, which may be acceptable.

### A fan-out to many subscribers should bound what it buffers for each one

Backpressure. When one producer broadcasts to many consumers, a consumer slower than the stream has a growing queue. If that queue is unbounded, one slow or stalled client grows memory until the process dies and takes every other client with it. The choices are to block (which stalls everyone), to drop or conflate (send only the latest value per key), or to disconnect the slow subscriber. Each of these is a decision, and an unbounded queue is the absence of one.

- A market-data server keeps an unbounded asyncio queue per SSE client. A client on a mobile connection stalls, its queue grows for an hour, and the task runs out of memory, dropping every subscriber.
- A chat server's websocket send buffers grow without limit for a client behind a stuck proxy.
- A log shipper buffers unboundedly while its destination is down, and fills the disk.
- Conflation works when only the latest value matters, like a price. It does not work for a stream of distinct trades, where dropping loses information, and a disconnect is more honest.

### A fallback should not share a cause of failure with the thing it backs up

Common-mode failure and failure-domain independence. Redundancy only helps against failures the replicas do not share. A backup path that depends on the same DNS, the same CDN configuration, the same upstream platform, the same account or the same deploy pipeline as the primary fails together with it in exactly the incidents it exists for. Listing the shared dependencies is cheap. Removing them may not be, and accepting one is a legitimate choice once it is visible.

- A status page is hosted on the same cloud region as the service it reports on, and it goes down with it.
- A backup notices site is fed from the same publishing platform as the main site, so if that platform is the cause of the outage, the backup stops updating at the moment it is needed.
- Two redundant links run through the same street cabinet.
- A disaster-recovery database replicates from the primary and faithfully replicates a destructive bug.
- If the fallback only ever needs to serve content already published, freezing during an upstream outage may be fine, provided it says when it was last updated.

## More rules, titles only

These were rated just as highly. Their stories are left for you to work out, which is the exercise.

- Messages and API requests should be safe to receive twice
- Older versions should refuse newer data with a clear message
- Black-box tests should run against the packaged artifact
- Client-side errors should carry the server's request ID
- Data should go to stdout, and progress and messages to stderr
- Cancelling in the UI should cancel the work on the server
- Firmware and settings updates should survive power loss part-way through
- Autoscaling should be capped at what shared dependencies can handle
- Cache keys should include everything the cached value depends on
- Scheduled runs should usually each process a named time window
- Service liveness checks should not generally include dependency liveness
- Reads that must agree with each other should come from one consistent snapshot
- Event records should store when the event happened, separately from when it was processed
- Code should not rely on any order among concurrent requests
- Each failure should be retried at only one layer of a call chain
