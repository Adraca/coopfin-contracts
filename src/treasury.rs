@@ -1,6 +1,7 @@
 use soroban_sdk::{contract, contractimpl, Env, Vec, map, symbol_short, symbol};
 use crate::errors::TreasuryError;

+const TTL_LEDGERS: u32 = 1000;
@@ -25,6 +26,10 @@ impl TreasuryContract {
             .push_back(ContributionRecord {
                 amount: amount,
                 timestamp: env.ledger().timestamp(),
+                // Bump TTL after writing history
+                env.storage().instance().extend_ttl(TTL_LEDGERS);
+                // Bump instance storage TTL at start of every state-changing function
+                Self::bump_instance(&env);
             });
         }
     }
@@ -33,6 +38,13 @@ impl TreasuryContract {
         Ok(())
     }
 }
+
+impl TreasuryContract {
+    fn bump_instance(env: &Env) {
+        env.storage().instance().extend_ttl(TTL_LEDGERS);
+    }
+}
+
--- a/src/voting.rs