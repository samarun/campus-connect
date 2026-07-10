import type { Metadata, Viewport } from "next";
import "@livekit/components-styles";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Campus Connect",
    template: "%s | Campus Connect"
  },
  description: "Configurable multisite video and program relay platform.",
  manifest: "/manifest.json"
};

export const viewport: Viewport = {
  themeColor: "#10252f",
  width: "device-width",
  initialScale: 1
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" data-theme="dark" suppressHydrationWarning>
      <body>
        <script
          dangerouslySetInnerHTML={{
            __html:
              "try{var t=localStorage.getItem('campus-connect-theme');if(t==='light'){document.documentElement.dataset.theme='light';document.documentElement.style.colorScheme='light'}else{document.documentElement.dataset.theme='dark';document.documentElement.style.colorScheme='dark'}}catch(e){}"
          }}
        />
        {children}
      </body>
    </html>
  );
}
