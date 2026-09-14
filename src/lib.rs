@@ -1,5 +1,7 @@
 use soroban_sdk::{contract, contractimpl, Env};

+const TTL_LEDGERS: u32 = 1000;
+
 #[contract]
 pub struct CoopFinanceContract;

@@ -8,6 +10,13 @@ impl CoopFinanceContract {
         // Contract implementation
     }
 
+    fn bump_instance(&self, env: Env) {
+        // Documentation: Bump instance storage TTL at start of every state-changing function
+        // Chosen TTL value of 1000 ledgers to ensure data persistence
+        // across typical usage patterns without excessive storage costs.
+        env.storage().instance().extend_ttl(TTL_LEDGERS);
+    }
+
     // Other contract methods...
 }
```

In each contract, the `extend_ttl` call is added after writing to persistent storage. A `bump_instance` utility function is added to bump the instance storage TTL, and it should be called at the start of every state-changing function. The chosen TTL value of 1000 ledgers is documented in comments.