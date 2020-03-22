# ignore_crate_experiments
Experimenting with Burntsushi's ignore crate for dir traversal.

How to go from a WalkDir entry into having everything stored as a hashmap as quickly as possible. 

All results are based on my own personal machine.

The channel method seems slightly quicker than the shared hashmap method.

Then I experimented with doing more work in the receiver and transmitter threads.
