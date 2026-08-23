# Goals

- Upgrade every UAR-owned @prometheus-ags/prometheus-entity-management dependency to the published 3.0.2 release and reconcile its lockfiles
- Reproduce and fix the installed chat Session Configuration sheet becoming non-responsive
- Correct Session Configuration sheet margin and padding around controls across supported viewport sizes
- Verify the fixes through the installed service at http://localhost:1906 using browser console/network evidence and server logs

## Analyze-stage goal amendments

The operator expanded the phase after the initial assessment by directing a deep
architecture review, requiring proper Entity Management use, and authorizing an
upstream Entity Management fix or GitHub issue.

- Preserve the original dependency goal as an immediate deliverable: move the UAR
  product from the `3.0.0-rc.1` workspace link to published 3.0.2 and reconcile the
  root and frontend lockfiles.
- In a parallel upstream track, fix the confirmed unbounded fetched-list publication
  defect in Prometheus Entity Management, commit/push/open its PR, and version the
  affected packages for the next patch release. If code cannot be changed safely,
  create an upstream issue with the observed notification-count reproduction.
- Replace the Session Configuration dead facade with canonical `AgentSession` state
  and prove a saved session model changes effective inference. Unsupported controls
  must be implemented end to end or removed; ignored fields do not satisfy the goal.
- Keep unsaved session business state explicit and inspectable in the entity graph,
  separate from committed session state.
