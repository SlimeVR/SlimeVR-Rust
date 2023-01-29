# Tips for Writing a Good Pull Request

I (@thebutlah) am a big fan of tiny PRs. In general, as the number of changes in a PR
increases, so too does the time it takes to review and discuss that PR, exponentially.

The biggest takeaway is that you should try to break features up into the smallest
individual components possible, and submit a PR for that as soon as is reasonable.
For me, that usually is at least once in a single session of coding. Its rare I have
a feature branch that sits on my fork. Sometimes its necessary, but thats like only
10% of the time.

If you want to hear more about my opinions or advice, you can expand the below. Or you
could go code or touch grass instead.

<details>
<summary>See a text wall...</summary>

## Ok But, Who Asked?
Why even care about this at all? The main reason is to be respectful of people's time,
both *your* time as well as others' time. 

If a PR is tiny, it makes it easy to merge. There are fewer lines to read and fewer
places where the PR could be blocked or stalled in review. This can happen when its not
clear what approach to take on a change, or there is a disagreement. 

If you submit small and incremental PRs, and PRs that are not mutually dependent on each
other, you will find that it becomes much faster and easier to get PRs merged.

## Understanding different types of changes in a PR

In general, PRs contain some combination of these types of changes:

* New features.
* New architectural approaches or code patterns, on top of which new features can later
  be built.
* Refactoring, like moving modules around, renaming variables, or anything tedius and
  mechanical, where we apply the same change in the same way to many spots in the code.
* Prototype or WIP code.

Most people will agree that new features are generally desired. When adding a new
feature, we often find that we need to do the other things on that list too. So the
natural way of submitting a PR will mix these all together.

However if you can make an active effort to compartmentalize stuff into smaller tasks
or components, it usually is quite straightforward to separate these things.

## A concrete example
Lets say I want to implement a new imu. To do this, I start prototyping the code in my
branch. As I prototype, I realize that I need some new `embedded-hal` peripheral, like
a `Delay`. To get access to this peripheral, I need to refactor the code to provide
that delay peripheral. Then I continue on my prototype. I add feature flags for the imu,
I write code that accesses the gyro, but I skip reading the accelerometer with a
`todo!()`, because I won't have time today to really start that. I also decide that it
would be better if we configured the choice of which imu to use via a config file
instead of cargo features.

Now on my local branch, I have the following bits of code all jumbled together across
several commits:
1. I started a WIP feature, the new imu. The gyro stuff definitely works, I can see that
  I'm getting data when I run it. But I cant actually finish the feature yet because of
  the accelerometer. Its also got a new feature flag, which means we can control if the
  code gets used or not.
2. I refactored the peripheral access to add a new delay field/function so that my imu
  can get the `Delay` peripheral.
3. Feature flags now use a config file instead of cargo features.

What is the dependency between these components?

* 1 depends on 2, clearly. I can't construct the IMU without 2.
* 1 isn't actually finished yet, but its feature flagged so its not going break anything
  if it gets merged. And the parts that aren't finished are clearly marked in the source
  code.
* 3 wasn't really needed by 1 or 2 at all. But it still seems useful.

Instead of lumping these all into a single PR, it will help speed the review process to
separate them. Submit a PR for *just* 2. Then submit another PR for *just* 1, which is
rebased on 2. Say "Rebased on 2" in the message of the PR. Submit another PR for 3.

The rationale is here:
* Why submit a PR for 1 even though its not finished? As long as its clear in the code
  with a `todo!()` what isn't done or needs changing, its better to submit it for
  review. This lets you get feedback from the reviewer early, because maybe there is
  something that they think needs to be changed or have an idea that could be helpful.
  Just be sure to note stuff in the PR description if there is anything unclear! Also, 
  merging it now in its WIP state is fine, because the code is feature flagged. Unless
  someone turns on that feature, its not going to even be run. It also means that 
  maybe someone else can go and implement the accelerometer access, now that they can
  build on top of that code. After all, Rome wasn't built in a day. You can say in
  the PR, "accel isn't implemented yet, but I tested gyro and that seems to work." so
  that the reviewer understands what actually they are supposed to be checking here.
* Why submit 2 separate from 1? 2 clearly is useful, and is a lot simpler than the work
  in 1. 1 may or may not take time to get reviewed or merged, and its a separate *thing*
  than 2. So by submitting 2 separately, you make it much easier to review both 1 and 2,
  because review times are usually exponential in the size of the PR. Also, maybe
  someone else is working on another imu that needs a delay too! They can use your
  change even while 1 is still being reviewed and workshopped.
* Why submit 3 separately? While its a useful change, and maybe makes implementing 1 a
  bit easier, it isn't really necessary for 1. 1 could be done using the old way of
  configuring features. Its also changing a pretty fundamental piece of the codebase's
  architecture, and something that people will probably want to discuss either on github
  or in discord. Its best to *ask* before making that type of change, or at the very
  least, make it in a separate PR so that if the reviewers don't agree with the approach
  or have a different opinion on the contents of the config file, it doesn't block 1 or
  2. These are often the *most* important types of changes to separate into their own
  PRs, because they have the highest probability of lengthy discussion.

Splitting these piceces into 3 prs is not difficult, especially if you realize early in
the process that you are really working on 3 things, not just 1. Always ask yourself,
could I implement this change in a separate PR, or does it really need to be bundled
with the current change? Maybe I could just make two chained PRs?

If you are unfamiliar with git, and aren't comfortable breaking commits up into separate
PRs and similar, ask us for help on discord! We are friendly and are happy to teach.
Worst case scenario, submit a PR and say "help I don't know how to break it up". The
other contributors might be able to help :)

</details>
