use axum::response::{Html, IntoResponse};

pub async fn geolocate() -> impl IntoResponse {
    Html(
        r#"
<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Geo Demo</title></head>
<body>
  <pre id="out"></pre>

  <script type="module">
    let position_var = undefined;
    const out = document.getElementById('out');

    // Persist / reuse a per‑browser UUID
    let id = localStorage.getItem('browser_id');
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem('browser_id', id);
    }

    if (!navigator.geolocation) {
        out.textContent = 'Geolocation not supported';
      } else {

      navigator.geolocation.getCurrentPosition(
        async pos => {
          const payload = {
            id,
            lat:  pos.coords.latitude,
            lon:  pos.coords.longitude
          };
          position_var = JSON.stringify(payload, null, 2);
          out.textContent = JSON.stringify(payload, null, 2);
          await fetch('/geolocate', {           // <-- new route
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
          });
        },
        err => out.textContent = `Error: ${err.message}`
      );
      }
  </script>
</body>
</html>
"#,
    )
}

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
