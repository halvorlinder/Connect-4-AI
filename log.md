## Pre December 30th

The minmax agent plays quite well. 
The fast eval takes about 200ns, while the original takes about 560ns.

## January 2nd 

After introducing game globals, the fast eval time is down to about 115ns.

Getting the next move from the minmax agent ~ 140 ms without fast eval.

Using fast eval reduced next move to ~ 35 ms.

Sorting the moves reduced it even further to ~ 7 ms.

## January 3rd

Applying symmetry pruning reduced next move to ~ 3.5ms