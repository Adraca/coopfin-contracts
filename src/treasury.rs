@@ -1,6 +1,7 @@
 use soroban_sdk::{contract, contractimpl, vec, Env, Symbol};
 use crate::storage_types::{ContributionRecord, ContributionMap};

+const TTL_LEDGERS: u32 = 1000;
@@ -23,6 +24,9 @@ impl TreasuryContract {
             env.storage().persistent().set(&contribution_key, &record);
 
             // Bump TTL after writing history
+            // Documentation: Chosen TTL value of 1000 ledgers to ensure data persistence
+            // across typical usage patterns without excessive storage costs.
+            env.storage().persistent().extend_ttl(&contribution_key, TTL_LEDGERS);
         }
     }
 }

--- a/src/voting.rs