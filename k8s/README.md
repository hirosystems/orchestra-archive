# Stacks Testnet Infrastructure

## API and Follower Blue/Green Deployments

The script in `scripts/api-bluegreen.sh` can be used to:

* Determine the current environment (blue/green) deployed for the API/follower-events
* Update the LOCAL K8s files to prepare for switching environments
* Update the LOCAL K8s files to perform an environment switch

This script makes **LOCAL** changes to the appropriate K8s resources in order to prepare for or perform an environment switch (blue -> green / green -> blue) for the API.
Once the LOCAL changes are made, it is up to the person executing this script to manually commit them to the staging branch in the k8s repo.

If the one making these changes does not have access to push changes into the staging branch, PRs are welcome.
After the change(s) make their way into the staging branch, ArgoCD will pick up these changes and deploy them within 5 minutes.

### 1. Check which environment is currently serving live traffic
This command will simply check the API Ingress resource in our K8s cluster to determine which environment is live.

**This command may change your currently selected `kubectl` context in order to query the correct cluster**

```bash
./scripts/api-bluegreen.sh -c

-> Switched to context "gke_ops-shared_us-east4-a_platform-core-001".
-> 2021-01-22T20:01:41.3NZ | INFO: LIVE ENVIRONMENT: green
```

### 2. Prepare for switching environments
This command will edit some LOCAL K8s files to effectively scale-up the environment which isn't serving live traffic. You should also make any changes to the environment being scaled up at this time (e.g. changing image versions, environment variables, resources, etc).

After running this command, these change(s) must be committed/merged into the staging branch in the k8s repo for ArgoCD to deploy them.

**This command may change your currently selected `kubectl` context in order to query the correct cluster**

```bash
./scripts/api-bluegreen.sh -p

-> Switched to context "gke_ops-shared_us-east4-a_platform-core-001".
-> 2021-01-22T20:11:46.3NZ | INFO: Local changes to the API and follower-evenets Statefulsets have been made.
-> 2021-01-22T20:11:46.3NZ | INFO: Review the changes locally with 'git diff' and commit to the 'staging' branch or create a PR against the staging branch when ready.
-> 2021-01-22T20:11:46.3NZ | INFO: Once committed/merged into the staging branch, ArgoCD will auto-deploy the changes within 5 minutes.
```

### 3. Check the preview environment
You might want to run this after ArgoCD deploys the changes above from preparing to switch environments.

Once the preview environment is sufficiently caught up with the stacks network, it should be safe to run the final command below to switch enviornments.

```bash
./api-bluegreen.sh -r

-> Switched to context "gke_ops-shared_us-east4-a_platform-core-001".
-> BLUE API AND FOLLOWER-EVENTS STATUS
-> stacks-node-api-blue-0 proxy block height: 865
-> stacks-node-api-blue-0 API block height: 800 # Indicates the blue envrionment is not yet caught up to the network
-> 
-> GREEN API AND FOLLOWER-EVENTS STATUS
-> stacks-node-api-green-0 proxy block height: 865
-> stacks-node-api-green-0 API block height: 865
->
-> pod "check-api" deleted
```

### 4. Switch environments
This command will edit a LOCAL K8s file to effectively switch the live environment.

After switching environments, you should verify everything works as expected before moving on to the next step to scale down the inactive environment.

After running this command, these change(s) must be committed/merged into the staging branch in the k8s repo for ArgoCD to deploy them.

**DO NOT run the command in step 5 until this change has been committed and deployed by ArgoCD. Otherwise you risk scaling down the environment you're trying to switch to.**

**This command may change your currently selected `kubectl` context in order to query the correct cluster**

```bash
./scripts/api-bluegreen.sh -s

-> Switched to context "gke_ops-shared_us-east4-a_platform-core-001".
-> 2021-01-22T20:11:49.3NZ | INFO: Local changes to the API and follower-evenets Statefulsets have been made.
-> 2021-01-22T20:11:49.3NZ | INFO: Review the changes locally with 'git diff' and commit to the 'staging' branch or create a PR against the staging branch when ready.
-> 2021-01-22T20:11:49.3NZ | INFO: Once committed/merged into the staging branch, ArgoCD will auto-deploy the changes within 5 minutes.
```

### 5. Scale down inactive environment
This command will edit some LOCAL K8s files to scale down the inactive environment.

Run this command after you verify the environment you switched to in the previous step is working as expected.

After running this command, these change(s) must be committed/merged into the staging branch in the k8s repo for ArgoCD to deploy them.
**This command may change your currently selected `kubectl` context in order to query the correct cluster**

```bash
./scripts/api-bluegreen.sh -d

-> Switched to context "gke_ops-shared_us-east4-a_platform-core-001".
-> 2021-01-22T20:11:49.3NZ | INFO: Local changes to the API and follower-evenets Statefulsets have been made.
-> 2021-01-22T20:11:49.3NZ | INFO: Review the changes locally with 'git diff' and commit to the 'staging' branch or create a PR against the staging branch when ready.
-> 2021-01-22T20:11:49.3NZ | INFO: Once committed/merged into the staging branch, ArgoCD will auto-deploy the changes within 5 minutes.
```
