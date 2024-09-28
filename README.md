# ONVP API Backend

The ONVP website uses an API which orchestrates member management, work group management, etc. It allows members to log
in to the website, access pages, take part of work groups, etc. To start the backend, the following is required:

* The database in use should be a PostgreSQL instance
* The keys for One-Time-Passwords and JWT should be generated using the <code>onvp-otp-keygen</code> / <code>
  onvp-jwt-keygen</code> commands which can be build using Cargo.
* The environment needs to be set up accordingly. A <code>.env</code> file can be used and will be read automatically. A
  template can be found in <code>.env.template</code>
* The backend should be started with the <code>onvp-backend</code> command, further de jobs should also be running.

