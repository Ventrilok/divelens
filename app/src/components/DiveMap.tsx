// Map of the dive site, under its GPS position (spec F5). Map tiles come from
// OpenFreeMap (OpenStreetMap data) and need Internet: without it, a message
// stands in for the map. Loaded on first use, the map library is large.

import { useEffect, useRef, useState } from "react";
import { Map as MapLibre, Marker, NavigationControl, setWorkerUrl } from "maplibre-gl";
import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url";
import "maplibre-gl/dist/maplibre-gl.css";
import { openUrl } from "@tauri-apps/plugin-opener";
import { t } from "../i18n";

const STYLE = "https://tiles.openfreemap.org/styles/fiord";
// About 20 km across: the island or the coast around the site.
const ZOOM = 11;
// The app's accent cyan: the chosen place.
const MARKER = "#69b7bf";

setWorkerUrl(workerUrl);

export default function DiveMap({ lat, lon }: { lat: number; lon: number }) {
  const box = useRef<HTMLDivElement>(null);
  const map = useRef<MapLibre | null>(null);
  const marker = useRef<Marker | null>(null);
  const failed = useRef(false);
  const [offline, setOffline] = useState(false);

  useEffect(() => {
    const el = box.current!;
    const m = new MapLibre({
      container: el,
      style: STYLE,
      center: [lon, lat],
      zoom: ZOOM,
      // Map data credits only (no library link): shown, then folded into an
      // "i" button once the map is moved.
      attributionControl: { compact: true },
      // The detail panel scrolls: the wheel scrolls it, the buttons zoom.
      scrollZoom: false,
      dragRotate: false,
      pitchWithRotate: false,
      touchPitch: false,
      locale: {
        "NavigationControl.ZoomIn": t("map.zoomIn"),
        "NavigationControl.ZoomOut": t("map.zoomOut"),
        "Map.Title": t("map.title"),
      },
    });
    m.touchZoomRotate.disableRotation();
    m.addControl(new NavigationControl({ showCompass: false }), "top-right");
    marker.current = new Marker({ color: MARKER }).setLngLat([lon, lat]).addTo(m);
    const fail = (value: boolean) => {
      failed.current = value;
      setOffline(value);
    };
    // Without Internet the style cannot load, and nothing shows.
    m.on("error", () => {
      if (!m.isStyleLoaded()) fail(true);
    });
    m.on("load", () => fail(false));
    const retry = () => failed.current && m.setStyle(STYLE);
    window.addEventListener("online", retry);
    // Credits link out to the browser, never inside the app's window.
    const links = (e: MouseEvent) => {
      const a = (e.target as Element).closest("a[href^='http']");
      if (!a) return;
      e.preventDefault();
      void openUrl(a.getAttribute("href")!);
    };
    el.addEventListener("click", links);
    map.current = m;
    return () => {
      window.removeEventListener("online", retry);
      el.removeEventListener("click", links);
      m.remove();
      map.current = null;
    };
    // Created once; a new dive moves the same map (below).
  }, []);

  useEffect(() => {
    const m = map.current;
    if (!m) return;
    m.jumpTo({ center: [lon, lat], zoom: ZOOM });
    marker.current?.setLngLat([lon, lat]);
    if (failed.current) m.setStyle(STYLE);
  }, [lat, lon]);

  return (
    <div className="map">
      <div ref={box} className="map-canvas" />
      {offline && <p className="map-offline">{t("map.offline")}</p>}
    </div>
  );
}
