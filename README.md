# Event Store Cloud CLI

`esc` is a command-line tool that allows you to access Event Store Cloud API in the comfort of your terminal.

## Authentication

`esc` supports several authentication methods. When making an authenticated call it resolves
credentials in a fixed order and uses the **first one available**:

1. **Service Account** — `ESC_CLIENT_ID` / `ESC_CLIENT_SECRET` environment variables.
2. **`--refresh-token` flag** — explicit refresh token (overrides a stored login).
3. **Browser login (PKCE)** — token stored by `esc login`.
4. **Legacy** — email/password (interactive prompt or `esc access tokens create`).

### Browser login (recommended)

Log in interactively through your browser:

```
esc login
```

This runs a browser-based PKCE OAuth flow and stores the resulting token. `esc` refreshes it
automatically; you don't need to do anything until the refresh token itself expires.

To remove the stored login token:

```
esc logout
```

### Service Account (machine-to-machine)

Set both environment variables; `esc` uses the OAuth2 client-credentials flow. No prompt, and
nothing is written to disk:

```
export ESC_CLIENT_ID=<client-id>
export ESC_CLIENT_SECRET=<client-secret>
esc resources organizations list
```

This takes priority over all other methods, making it the preferred option for CI and automation.

### Legacy email/password

If you have neither a Service Account nor a stored login token, `esc` falls back to interactively
asking your email and password. For a non-interactive variant:

```
esc access tokens create --email <email> --unsafe-password <password>
```

Rest assured that `esc` doesn't store your password in your system.

## Scripting / Continuous Integration (CI) Usage

For CI, prefer **Service Account** credentials via `ESC_CLIENT_ID` / `ESC_CLIENT_SECRET` (see above) —
nothing touches the filesystem.

Alternatively, `esc` exposes a `--refresh-token=<your refresh token>` parameter. If set, `esc` won't
rely on the filesystem to fetch your refresh token, and the refresh token won't be persisted on the
filesystem either. An explicit `--refresh-token` overrides a stored browser-login token.

## Implicit parameters

Virtually all commands require `--org-id` and `--project-id` parameters. It is possible to tell
`esc` to use a preset `--org-id` or `--project-id`. You only need to create a local profile.

```
esc profiles set --profile <profile> --name <name> --value <value>
```

For example if you want to set a default `--org-id` for a profile named `my_profile`:

```
esc profiles set --profile my_profile --name org-id --value <my-org-id>
```

Similarly, if you want to set a default `--project-id` do:

```
esc profiles set --profile my_profile --name project-id --value <my-project-id>
```

Don't forget to set your local `my_profile` profile to be the default profile by doing the following:

```
esc profiles default set --value my_profile
```

From now, all the commands that need `--org-id` or `--project-id` will pick the value set in your
`my_profile` profile.

You can find more information about `profiles` by entering:

```
esc profiles --help
```

## Output Formats

This tool has historically shown output using it's own custom format instead of what the API returns. This will be deprecated in the future.

To view all the data returned from the API, pass `--fmt api`.

It is possible to tell `esc` to always use this format by setting it in your profile:

```
esc profiles set --profile my_profile --name fmt --value api
```

## JSON commands output rendering

You can render any read command output in JSON by using the `--json` flag.

```
esc resources organizations list --json
```

## Shell completions

You can generate shell completion script by using the `generate-{shell}-completion` command. Currently supported:

- Bash
- Zsh
- Powershell

Additional shells can be supported, please open a feature request.

The content of the script is displayed on STDOUT.

Example:

```
esc generate-bash-completion > /usr/share/bash-completion/completions/esc.bash
```

## Common usage examples:

### Create a network.

```
esc infra networks create  --cidr-block <cidr-block> --description <description> --provider <provider> --region <region>
```

You can find out more about each option by entering:

```
esc infra networks create --help
```

### Create a peering link.

```
esc infra peerings create --org-id <org-id> --project-id <project-id> --description <description> --peer-account-id <peer-account-id> --peer-network-id <peer-network-id> --peer-network-region <peer-network-region>

```

You can find out more about each option by entering:

```
esc infra peerings create --help
```

### Create a cluster.

```
esc mesdb clusters create --org-id <org-id> --project-id <project-id> --description <description> --disk-size-in-gb <disk-size-in-gb> --disk-type <disk-type> --instance-type <instance-type> --network-id <network-id> --projection-level <projection-level> --server-version <server-version> --topology <topology>
```

You can find out more about each option by entering:

```
esc mesdb clusters create --help
```

### Stop a cluster

```
esc mesdb clusters stop --org-id <org-id> --project-id <project-id> --id <cluster-id>
```

### Start a cluster

```
esc mesdb clusters start --org-id <org-id> --project-id <project-id> --id <cluster-id>
```

### Create a shared cluster

```
esc mesdb shared-clusters create --deployment-tier s0 --name <name> --projection-level <projection-level> --provider <provider> --region <region> --server-version <server-version> --topology <topology> --acl-id <acl-id>
```

### Create a refresh token.

(example: for use with terraform)

```
esc access tokens create --email <email>
```

You can display your current refresh token with:

```
esc access tokens display
```

### Update Organization MFA requirement

```
escp resources organizations update-mfa-status --enabled=true
```

### List members of an organization.

```
esc access members list
```

### Enable a member of an organization

```
esc access members update --id <member-id> --active true
```

### Disable a member of an organization

```
esc access members update --id <member-id> --active false
```

### Deletes a member from an organization

```
esc access members delete --id <member-id>
```
