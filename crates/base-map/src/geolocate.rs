use axum::response::{Html, IntoResponse};

pub async fn geolocate() -> impl IntoResponse {
    Html(
        r#"
<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Geo Demo</title></head>
<body>
  <div style="font-family: Arial, sans-serif; padding: 20px;">
    <h2>Location Service</h2>
    <div id="status"></div>
    <pre id="out"></pre>
  </div>

  <script type="module">
    const out = document.getElementById('out');
    const status = document.getElementById('status');

    // Persist / reuse a per‑browser UUID
    let id = localStorage.getItem('browser_id');
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem('browser_id', id);
    }

    async function checkLocationPermission() {
      if (!navigator.geolocation) {
        status.innerHTML = '<p style="color: red;">Geolocation is not supported by this browser.</p>';
        return false;
      }

      if (!navigator.permissions) {
        // Fallback for browsers without Permissions API
        return requestLocationDirectly();
      }

      try {
        const permission = await navigator.permissions.query({name: 'geolocation'});

        switch(permission.state) {
          case 'granted':
            status.innerHTML = '<p style="color: green;">Location permission granted. Getting location...</p>';
            return getCurrentLocation();
          case 'denied':
            status.innerHTML = '<p style="color: red;">Location permission denied. Please enable location access in your browser settings and refresh the page.</p>';
            return false;
          case 'prompt':
            status.innerHTML = '<p style="color: orange;">Requesting location permission...</p>';
            return requestLocationDirectly();
          default:
            return requestLocationDirectly();
        }
      } catch (error) {
        console.error('Error checking permission:', error);
        return requestLocationDirectly();
      }
    }

    function requestLocationDirectly() {
      status.innerHTML = '<p>Requesting location access...</p>';
      return getCurrentLocation();
    }

    function getCurrentLocation() {
      return new Promise((resolve, reject) => {
        navigator.geolocation.getCurrentPosition(
          async pos => {
            const payload = {
              id,
              lat: pos.coords.latitude,
              lon: pos.coords.longitude,
              accuracy: pos.coords.accuracy,
              timestamp: pos.timestamp
            };

            out.textContent = JSON.stringify(payload, null, 2);
            status.innerHTML = '<p style="color: green;">Location obtained successfully!</p>';

            try {
              await fetch('/geolocate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
              });
              status.innerHTML += '<p style="color: green;">Location sent to server.</p>';
            } catch (fetchError) {
              status.innerHTML += `<p style="color: orange;">Warning: Could not send location to server: ${fetchError.message}</p>`;
            }

            resolve(true);
          },
          err => {
            handleLocationError(err);
            reject(err);
          },
          {
            enableHighAccuracy: true,
            timeout: 15000,
            maximumAge: 60000
          }
        );
      });
    }

    function handleLocationError(err) {
      let errorMessage = '';
      let color = 'red';

      switch(err.code) {
        case err.PERMISSION_DENIED:
          errorMessage = 'Location access denied. Please enable location access in your browser settings and refresh the page.';
          break;
        case err.POSITION_UNAVAILABLE:
          errorMessage = 'Location information is unavailable. Please check your GPS/location services.';
          color = 'orange';
          break;
        case err.TIMEOUT:
          errorMessage = 'Location request timed out. Please refresh the page to try again.';
          color = 'orange';
          break;
        default:
          errorMessage = `Unknown error occurred: ${err.message}`;
          break;
      }

      status.innerHTML = `<p style="color: ${color};">Error: ${errorMessage}</p>`;
      out.textContent = `Error: ${errorMessage}`;
    }

    // Start the location check when page loads
    checkLocationPermission();
  </script>
</body>
</html>
"#,
    )
}

// v2
// pub async fn geolocate() -> impl IntoResponse {
//     Html(
//         r#"
// <!doctype html>
// <html lang="en">
// <head><meta charset="utf-8"><title>Geo Demo</title></head>
// <body>
//   <pre id="out"></pre>
//
//   <script type="module">
//     let position_var = undefined;
//     const out = document.getElementById('out');
//
//     // Persist / reuse a per‑browser UUID
//     let id = localStorage.getItem('browser_id');
//     if (!id) {
//       id = crypto.randomUUID();
//       localStorage.setItem('browser_id', id);
//     }
//
//     if (!navigator.geolocation) {
//         out.textContent = 'Geolocation not supported';
//       } else {
//
//       navigator.geolocation.getCurrentPosition(
//         async pos => {
//           const payload = {
//             id,
//             lat:  pos.coords.latitude,
//             lon:  pos.coords.longitude
//           };
//           position_var = JSON.stringify(payload, null, 2);
//           out.textContent = JSON.stringify(payload, null, 2);
//           await fetch('/geolocate', {           // <-- new route
//             method: 'POST',
//             headers: { 'Content-Type': 'application/json' },
//             body: JSON.stringify(payload)
//           });
//         },
//         err => out.textContent = `Error: ${err.message}`
//       );
//       }
//   </script>
// </body>
// </html>
// "#,
//     )
// }

// v1
// pub async fn geolocate() -> impl IntoResponse {
//     // A minimal page that asks only after the user clicks.
//     Html(r#"
// <!doctype html>
// <html lang="en">
// <head><meta charset="utf-8"><title>Geo Demo</title></head>
// <body>
//   <pre id="out"></pre>
//
//   <script>
//     console.log('Hello from the browser');
//     const out = document.getElementById('out');
//     if (!navigator.geolocation) {
//       out.textContent = 'Geolocation not supported';
//     } else {
//       navigator.geolocation.getCurrentPosition(
//         pos => out.textContent =
//           `Lat${pos.coords.latitude}, Lon${pos.coords.longitude}`,
//         err => out.textContent = `Error: ${err.message}`
//       );
//     }
//   </script>
// </body>
// </html>
// "#)
// }
