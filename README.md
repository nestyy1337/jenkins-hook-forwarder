Mapping from TOML into build triggers.
Listens at localhost so ideally there is webhook forwarder like smee.


Example:

[jenkins]
url = "https://jenkins.domain.com"
port = 443
username = "api-trigger"
api =  "xxxx"

[repos]

[repos."repo_name"]
branch_job_mapping = { "branch_name" = "build_name", "branch_name2" = "build_name2" }

