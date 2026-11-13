# Remediate F14 offline source dependencies

The signed release includes a full recursive offline source archive. Its recursive OSV scan failed on affected dependencies across the UAR tree and nested repositories. Integrate dependency fixes in their owning sources and retain the archive, the offline minimal build, and the existing scanner policy. No published artifact changes in this child.
