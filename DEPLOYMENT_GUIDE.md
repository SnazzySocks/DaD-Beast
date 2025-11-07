# 🚀 Easy Deployment Guide - DaD-Beast Tracker Platform

This guide will help you deploy both the backend and frontend of the DaD-Beast tracker platform, even if you have minimal technical knowledge.

## 📋 What You'll Need

Before you start, you'll need:

1. **A computer** running Linux, macOS, or Windows
2. **Docker Desktop** installed ([Download here](https://www.docker.com/products/docker-desktop))
3. **About 30 minutes** of time
4. **At least 4GB of free RAM** and 10GB of disk space

---

## 🎯 Quick Start (Recommended for Beginners)

This is the easiest way to get everything running with one command!

### Step 1: Download the Project

Open a terminal (Command Prompt on Windows, Terminal on Mac/Linux) and run:

```bash
# Download the project
git clone https://github.com/SnazzySocks/DaD-Beast.git

# Go into the project folder
cd DaD-Beast/tracker-platform
```

### Step 2: Set Up Your Configuration

Copy the example configuration file:

```bash
# On Mac/Linux:
cp .env.example .env

# On Windows (Command Prompt):
copy .env.example .env
```

### Step 3: 🔐 IMPORTANT - Set a Secure Password

You MUST change the default security key before running the application:

**On Mac/Linux:**
```bash
# Generate a random secure key automatically
export JWT_SECRET=$(openssl rand -base64 32)
sed -i "s/change-me-in-production-min-32-characters-long-secret-key/$JWT_SECRET/g" .env
```

**On Windows (PowerShell):**
```powershell
# Generate a random secure key
$JWT_SECRET = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 32 | % {[char]$_})

# Update the .env file
(Get-Content .env) -replace 'change-me-in-production-min-32-characters-long-secret-key', $JWT_SECRET | Set-Content .env
```

**OR manually:**
1. Open the `.env` file in a text editor
2. Find the line: `APP__AUTH__JWT_SECRET=change-me-in-production-min-32-characters-long-secret-key`
3. Replace `change-me-in-production-min-32-characters-long-secret-key` with a random string of at least 32 characters

### Step 4: Start Everything! 🎉

Run this single command to start all services:

```bash
docker-compose up -d
```

This will:
- Download all necessary components (first time only, takes 5-10 minutes)
- Start the database (PostgreSQL)
- Start the cache (Redis)
- Start the search engine (Meilisearch)
- Start the tracker application
- Start the monitoring tools (Prometheus & Grafana)

### Step 5: Check if It's Working ✅

After a minute or two, open your web browser and visit:

**Main Application:**
- http://localhost:8080/health

You should see a response like: `{"status":"healthy"}`

**Monitoring Dashboard:**
- http://localhost:3001 (Grafana - username: `admin`, password: `admin`)

**Metrics:**
- http://localhost:9091 (Prometheus)

---

## 🎨 Frontend Setup (Optional)

The backend is now running! If you also want to run the frontend development server:

### Step 1: Install Node.js

Download and install Node.js 20 LTS from: https://nodejs.org/

### Step 2: Set Up Frontend

```bash
# Go to the frontend folder
cd frontend

# Copy the example environment file
cp .env.example .env

# Install dependencies (first time only)
npm install

# Start the development server
npm run dev
```

The frontend will be available at: http://localhost:5173

---

## 🛠️ Common Commands

### Starting the Application
```bash
# Start all services
docker-compose up -d

# Start and see logs in real-time
docker-compose up
```

### Stopping the Application
```bash
# Stop all services
docker-compose down

# Stop and remove all data (fresh start)
docker-compose down -v
```

### Viewing Logs
```bash
# See logs from all services
docker-compose logs -f

# See logs from specific service
docker-compose logs -f app
docker-compose logs -f postgres
```

### Checking Service Status
```bash
# List all running services
docker-compose ps

# Check application health
curl http://localhost:8080/health
```

---

## 🔧 Troubleshooting

### Problem: Port Already in Use

**Error:** `Bind for 0.0.0.0:8080 failed: port is already allocated`

**Solution:** Another application is using port 8080. Either:
1. Stop the other application, OR
2. Change the port in `.env`:
   ```
   APP__SERVER__PORT=8081
   ```
   And in `docker-compose.yml`, change:
   ```yaml
   ports:
     - "8081:8080"  # Change first number to match .env
   ```

### Problem: Docker Not Running

**Error:** `Cannot connect to the Docker daemon`

**Solution:**
1. Open Docker Desktop
2. Wait for it to fully start (whale icon in system tray)
3. Try your command again

### Problem: Database Connection Failed

**Error:** `connection refused` or `database unavailable`

**Solution:**
```bash
# Restart the database
docker-compose restart postgres

# Wait 30 seconds, then check logs
docker-compose logs postgres
```

### Problem: Out of Memory

**Error:** `Killed` or `OOMKilled`

**Solution:**
1. Open Docker Desktop
2. Go to Settings → Resources
3. Increase Memory to at least 4GB
4. Click "Apply & Restart"

---

## 📊 What Each Service Does

When you run `docker-compose up`, these services start:

| Service | Purpose | Port | URL |
|---------|---------|------|-----|
| **app** | Main tracker application | 8080 | http://localhost:8080 |
| **postgres** | Database (stores all data) | 5432 | Internal only |
| **redis** | Cache (makes things fast) | 6379 | Internal only |
| **meilisearch** | Search engine | 7700 | http://localhost:7700 |
| **prometheus** | Metrics collection | 9091 | http://localhost:9091 |
| **grafana** | Monitoring dashboards | 3001 | http://localhost:3001 |

---

## 🌐 Accessing Your Tracker

Once everything is running:

### API Endpoints

Test the API with these URLs (use a web browser or tools like Postman):

**Health Check:**
```
GET http://localhost:8080/health
```

**Tracker Announce (for BitTorrent clients):**
```
GET http://localhost:8080/tracker/announce
```

**GraphQL Playground (interactive API explorer):**
```
http://localhost:8080/graphql/playground
```

**Search Torrents:**
```
GET http://localhost:8080/api/v1/search/torrents?q=ubuntu
```

### Example: Register a User

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePassword123!"
  }'
```

---

## 🚀 Production Deployment

**⚠️ IMPORTANT:** The quick start method is for development only!

For production deployment, you MUST:

### Security Checklist

1. **Change ALL default passwords:**
   ```bash
   # In .env file:
   APP__AUTH__JWT_SECRET=<generate-32-char-random-string>

   # In docker-compose.yml:
   POSTGRES_PASSWORD=<generate-strong-password>
   redis password:<generate-strong-password>
   MEILI_MASTER_KEY=<generate-strong-password>
   ```

2. **Enable HTTPS:**
   - Get an SSL certificate (Let's Encrypt is free)
   - Configure reverse proxy (Nginx or Traefik)

3. **Configure environment:**
   ```bash
   APP__TELEMETRY__ENVIRONMENT=production
   APP__TELEMETRY__LOG_FORMAT=json
   ```

4. **Set up backups:**
   ```bash
   # Backup database daily
   docker exec tracker-platform-postgres-1 pg_dump -U tracker tracker > backup.sql
   ```

5. **Use environment variables for secrets:**
   - Never commit `.env` to git
   - Use secret management (AWS Secrets Manager, HashiCorp Vault)

6. **Resource limits:**
   - Monitor CPU/RAM usage
   - Set up alerts in Grafana
   - Scale services as needed

---

## 📚 Next Steps

Now that your tracker is running:

1. **Read the Architecture** - See [ARCHITECTURE.md](./ARCHITECTURE.md) for system design
2. **Explore Features** - Check [FEATURE_COMPARISON.md](./FEATURE_COMPARISON.md) to see what's included
3. **Read the API Docs** - Visit http://localhost:8080/graphql/playground
4. **Check the Tests** - See [tracker-platform/TESTING.md](./tracker-platform/TESTING.md)
5. **Monitor Performance** - Open Grafana at http://localhost:3001

---

## 🆘 Getting Help

If you're stuck:

1. **Check the logs:** `docker-compose logs -f`
2. **Search GitHub Issues:** [DaD-Beast Issues](https://github.com/SnazzySocks/DaD-Beast/issues)
3. **Read the detailed README:** [tracker-platform/README.md](./tracker-platform/README.md)
4. **Check Docker status:** `docker-compose ps`

---

## 🎓 Learning Resources

New to Docker or BitTorrent trackers? Check these out:

- **Docker Tutorial:** https://docs.docker.com/get-started/
- **What is a BitTorrent Tracker:** https://en.wikipedia.org/wiki/BitTorrent_tracker
- **PostgreSQL Basics:** https://www.postgresqltutorial.com/
- **Rust Programming:** https://www.rust-lang.org/learn

---

## 📝 Summary

**To start everything:**
```bash
cd DaD-Beast/tracker-platform
docker-compose up -d
```

**To check if it's working:**
```bash
curl http://localhost:8080/health
```

**To stop everything:**
```bash
docker-compose down
```

That's it! You now have a fully functional BitTorrent tracker platform running locally. 🎉

---

**Version:** 1.0
**Last Updated:** 2025-11-07
**Difficulty:** Beginner-Friendly ⭐
