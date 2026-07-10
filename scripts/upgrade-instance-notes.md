# Upgrade from c7i.2xlarge to c7i.4xlarge

Use this when CPU, RTC packet loss, or stream relay encode/playback load is consistently high during rehearsal.

1. Announce a maintenance window.
2. Run `./scripts/backup.sh` and copy the `.tgz` to S3 or another workstation.
3. In AWS EC2, stop the instance.
4. Change instance type from `c7i.2xlarge` to `c7i.4xlarge`.
5. Start the instance and confirm the Elastic IP reattaches.
6. SSH in and run:

```bash
cd /opt/campus-connect
docker compose ps
./scripts/healthcheck.sh
```

7. Run a three-campus Greeting Mode rehearsal and an stream relay SRT test before service.

Rollback is the same process in reverse: stop the instance, change back to `c7i.2xlarge`, start, and run health checks.
