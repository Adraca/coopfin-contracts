@@ -1,6 +1,7 @@
 use soroban_sdk::{contract, contractimpl, Env, Symbol};
 use crate::storage_types::{Vote, VoteMap};

+const TTL_LEDGERS: u32 = 1000;
@@ -23,6 +24,9 @@ impl VotingContract {
             env.storage().persistent().set(&vote_key, &vote);
 
             // Bump TTL after writing vote map
+            // Documentation: Chosen TTL value of 1000 ledgers to ensure data persistence
+            // across typical usage patterns without excessive storage costs.
+            env.storage().persistent().extend_ttl(&vote_key, TTL_LEDGERS);
         }
     }
 }

--- a/src/lib.rs
