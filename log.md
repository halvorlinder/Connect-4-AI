## Pre December 30th

The minmax agent plays quite well. 
The fast eval takes about 200ns, while the original takes about 560ns.

## January 2nd 

After introducing game globals, the fast eval time is down to about 115ns.

Getting the next move from the minmax agent ~ 140 ms without fast eval.

Using fast eval reduced next move to ~ 35 ms.