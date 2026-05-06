use dioxus::prelude::*;

use crate::Scheme;

const COOKIE: &str = "dx_theme";
const CHANNEL: &str = "dx-theme";

pub fn theme_seed() {
    let _ = document::eval(&format!(
        r#"
        (function () {{
          if (window.__dx_theme_seeded) return;
          window.__dx_theme_seeded = true;

          const COOKIE = '{COOKIE}';
          const CHANNEL = '{CHANNEL}';

          function read() {{
            const parts = document.cookie.split(';');
            for (let p of parts) {{
              p = p.trim();
              if (p.startsWith(COOKIE + '=')) return decodeURIComponent(p.slice(COOKIE.length + 1));
            }}
            return null;
          }}

          function apply(theme) {{
            if (theme === 'dark' || theme === 'light') {{
              document.documentElement.setAttribute('data-theme', theme);
            }} else {{
              document.documentElement.removeAttribute('data-theme');
            }}
          }}

          apply(read());

          try {{
            const ch = new BroadcastChannel(CHANNEL);
            ch.addEventListener('message', (event) => apply(event.data && event.data.theme));
            window.__dx_theme_channel = ch;
          }} catch (_) {{}}
        }})();
        "#
    ));
}

pub fn set_scheme(scheme: Scheme) {
    let theme = match scheme {
        Scheme::Light => "light",
        Scheme::Dark => "dark",
        Scheme::System => "system",
    };

    let _ = document::eval(&format!(
        r#"
        (function () {{
          const COOKIE = '{COOKIE}';
          const CHANNEL = '{CHANNEL}';
          const theme = '{theme}';

          if (theme === 'dark' || theme === 'light') {{
            document.documentElement.setAttribute('data-theme', theme);
            document.cookie = COOKIE + '=' + theme + '; path=/; max-age=31536000; samesite=lax';
          }} else {{
            document.documentElement.removeAttribute('data-theme');
            document.cookie = COOKIE + '=; path=/; max-age=0; samesite=lax';
          }}

          try {{
            const ch = window.__dx_theme_channel;
            const payload = {{ theme }};
            if (ch && typeof ch.postMessage === 'function') {{
              ch.postMessage(payload);
            }} else {{
              const tmp = new BroadcastChannel(CHANNEL);
              tmp.postMessage(payload);
              tmp.close();
            }}
          }} catch (_) {{}}
        }})();
        "#
    ));
}

pub async fn read_cookie_scheme() -> Scheme {
    let mut eval = document::eval(&format!(
        r#"
        (function () {{
          const COOKIE = '{COOKIE}';
          const parts = document.cookie.split(';');
          for (let p of parts) {{
            p = p.trim();
            if (p.startsWith(COOKIE + '=')) {{
              dioxus.send(decodeURIComponent(p.slice(COOKIE.length + 1)));
              return;
            }}
          }}
          dioxus.send('system');
        }})();
        "#
    ));

    match eval.recv::<String>().await.as_deref() {
        Ok("light") => Scheme::Light,
        Ok("dark") => Scheme::Dark,
        _ => Scheme::System,
    }
}
