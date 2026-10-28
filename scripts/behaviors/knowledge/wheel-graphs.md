# Wheel graphs: sandpile group order

Source: Krisztián Dsupin and Szabolcs Tengely, *Discrete logarithm problem in some families of sandpile groups* (2020), section 3, https://arxiv.org/pdf/2011.08296 .

A wheel W_n here has n rim vertices joined in a cycle and one additional hub. The sandpile (critical) group's order equals the number of spanning trees. For odd n this order is L_n squared; for even n it is five times F_n squared, where L and F are the Lucas and Fibonacci numbers. The paper reports the wheel formula attributed to Biggs.

Computed example: L_0 = 2, L_1 = 1 and L_n = L_(n-1) + L_(n-2), so L_11 = 199. A wheel with 11 rim vertices and one hub therefore has sandpile group order 199 squared = 39601. This states the order, not an invariant-factor decomposition.

The profile label and retrieval seal appended by the behavior script are local validation metadata, not facts from the paper.
