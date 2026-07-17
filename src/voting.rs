@@ -1,6 +1,7 @@
 use soroban_sdk::{contract, contractimpl, Env, Vec, map, symbol_short, symbol};
 use crate::errors::VotingError;

+const TTL_LEDGERS: u32 = 1000;
@@ -25,6 +26,10 @@ impl VotingContract {
             .push_back(VoteRecord {
                 member: member,
                 choice: choice,
+                // Bump TTL after writing vote
+                env.storage().instance().extend_ttl(TTL_LEDGERS);
+                // Bump instance storage TTL at start of every state-changing function
+                Self::bump_instance(&env);
             });
         }
     }
@@ -33,6 +38,13 @@ impl VotingContract {
         Ok(())
     }
 }
+
+impl VotingContract {
+    fn bump_instance(env: &Env) {
+        env.storage().instance().extend_ttl(TTL_LEDGERS);
+    }
+}
+
```