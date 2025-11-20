# Operator Manual

## System Administration

### Service Management

#### Starting Services

Services are managed by `initd`. To start a service:

```bash
# Via IPC
ipc send initd start memoryd
```

#### Stopping Services

```bash
ipc send initd stop memoryd
```

#### Service Status

```bash
ipc send initd status memoryd
```

### Policy Management

#### Adding a Policy

```rust
let policy = Policy {
    name: "example_policy".to_string(),
    policy_type: PolicyType::Security,
    rules: vec![/* ... */],
    priority: 1,
    // ...
};

let policy_id = PolicyEngine::get().unwrap().add_policy(policy);
```

#### Updating a Policy

```rust
let new_version = PolicyEngine::get().unwrap()
    .update_policy(policy_id, updated_policy)?;
```

#### Enabling/Disabling Policies

```rust
PolicyEngine::get().unwrap()
    .set_policy_enabled(policy_id, true)?;
```

### Monitoring

#### Viewing Metrics

```rust
let metrics = ObservabilitySystem::get().unwrap()
    .collect_metrics();
```

#### Agent Profiles

```rust
let profile = ObservabilitySystem::get().unwrap()
    .get_agent_profile(agent_id)?;
```

### Audit Logs

#### Verifying Log Integrity

```rust
let is_valid = AuditLog::verify();
```

#### Querying Audit Events

```rust
let events = AuditManager::get_events(Some(filter));
```

## Troubleshooting

### Service Failures

1. Check service logs
2. Verify service dependencies
3. Check resource quotas
4. Review policy rules

### Performance Issues

1. Check system metrics
2. Review agent profiles
3. Check resource usage
4. Review scheduler statistics

### Security Issues

1. Review audit logs
2. Check policy violations
3. Verify capability grants
4. Review firewall rules

## Maintenance

### Backup

- Audit logs are automatically backed up
- Policy configurations should be backed up
- Service configurations should be backed up

### Updates

- Policy updates create new versions
- Services can be updated via IPC
- Kernel updates require reboot

## Best Practices

1. **Policy Management:**
   - Use versioning for policy changes
   - Test policies in staging
   - Monitor policy evaluation

2. **Service Management:**
   - Use health checks
   - Monitor service dependencies
   - Set appropriate restart policies

3. **Security:**
   - Review audit logs regularly
   - Keep policies up to date
   - Monitor security events

4. **Performance:**
   - Monitor system metrics
   - Review agent profiles
   - Optimize resource usage

