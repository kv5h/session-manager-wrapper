# session-manager-wrapper

A Rust wrapper for AWS SSM Session Manager

## Testing

> [!WARNING]
>
> Under maintenance

### Install LocalStack

```bash
pip3 install localstack
```

```bash
brew install localstack/tap/localstack-cli
```

#### (Option) List available services

```bash
localstack start -d
```

```bash
localstack status services
```

Output sample:

```plaintext
┏━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━┓
┃ Service                  ┃ Status      ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━┩
│ acm                      │ ✔ available │
│ apigateway               │ ✔ available │
│ cloudformation           │ ✔ available │
│ cloudwatch               │ ✔ available │
│ config                   │ ✔ available │
│ dynamodb                 │ ✔ available │
│ dynamodbstreams          │ ✔ available │
│ ec2                      │ ✔ available │
│ es                       │ ✔ available │
│ events                   │ ✔ available │
...
```

### Export environment variables

```bash
direnv allow
```

```bash
cp .env.example.toml .env.toml
```

### Setting up testing environment

**NOTE**: Run this after exporting the environment variables.

```bash
bash ./script/init_test_env.sh
```

```bash
export TEST_INSTANCE_ID=$(awslocal ec2 describe-instances \
  --filters "Name=image-id,Values=6db4dfa5-ea15-44d1-918b-cb39b7b6e1ca" \
  | jq -r .Reservations[0].Instances[0].InstanceId)
```

### Run test

```bash
cargo test
```

```bash
# With stdout
cargo test -- --nocapture
```

### Clean up

```bash
docker compose -f compose/compose.yaml down && unset TEST_INSTANCE_ID
```
