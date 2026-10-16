import { useState } from 'react';
import { createRoot } from 'react-dom/client';
import manifest from './rooms.json';
import './style.css';
import { RoomScene } from './RoomScene';
import { HallOfBrothers } from './HallOfBrothers';

const rooms = manifest.rooms;
type Room = (typeof rooms)[number];
type Detail = { title: string; text: string };

function House() {
  const [roomId, setRoomId] = useState('bedroom');
  const [detail, setDetail] = useState<Detail | null>(null);
  const [motionOn, setMotionOn] = useState(true);
  const room: Room = rooms.find((item) => item.id === roomId) ?? rooms[0];
  const go = (next: string) => {
    if (!rooms.some((item) => item.id === next)) return;
    setRoomId(next);
    setDetail(null);
    window.scrollTo({ top: 0, behavior: 'instant' });
  };
  const inspect = (title: string, text: string) => setDetail({ title, text });

  return (
    <div className="shell">
      <header className="masthead">
        <div className="brandmark" aria-hidden="true">∞</div>
        <div className="brandcopy">
          <p className="overline">INFINITE PORCH PRESENTS · AN ILLUSTRATIVE EXPERIENCE</p>
          <h1>The Infinite House</h1>
          <p className="tagline">A little piece of forever, somewhere in 1987.</p>
        </div>
        <span className="badge">LOCAL ART DEMO · NO LIVE CONNECTIONS</span>
      </header>

      <aside className="public-demo-banner" aria-label="Public demonstration safety notice">
        <strong>OPEN HOUSE · PUBLIC PREVIEW</strong>
        <span>Explore ten illustrated rooms and design sample Hall doors. This site does not connect to an AI or a private Porch. Hall drafts stay in this browser, not in a private or encrypted vault. Please don't enter confidential information.</span>
      </aside>

      <div className="house-grid">
        <nav className="directory" aria-label="House room directory">
          <p className="overline directory__heading">FLOOR PLAN / EXPLORE</p>
          {rooms.map((item, index) => (
            <button
              className={'room-link ' + (item.id === room.id ? 'room-link--active' : '')}
              aria-current={item.id === room.id ? 'page' : undefined}
              key={item.id}
              onClick={() => go(item.id)}
            >
              <span className="room-number">{String(index + 1).padStart(2, '0')}</span>
              <span>{item.name}</span>
              <span className="room-chevron" aria-hidden="true">↗</span>
            </button>
          ))}
          <p className="directory__note">Each room is a real navigation point; objects currently open explanatory previews.</p>
        </nav>

        <main className="main">
          <div className="chapter">
            <div>
              <p className="overline">{room.kicker}</p>
              <h2>{room.name}</h2>
              <p className="lede">{room.mood}</p>
            </div>
            <span className="chapter__count">{String(rooms.indexOf(room) + 1).padStart(2,'0')} / {rooms.length}</span>
          </div>

          <section className="stage" aria-label={room.name + ' illustrated scene'} data-room={room.id}>
            <RoomScene roomId={room.id} motionOn={motionOn}/>
            <div className="scene-controls"><button type="button" aria-pressed={motionOn} onClick={() => setMotionOn((current) => !current)}>{motionOn ? 'Pause ambient motion' : 'Enable ambient motion'}</button></div>
            <span className="scene-status">{room.name} · illustrated 1987</span>

            {room.id === 'hall-of-brothers' && <div className="hall-threshold-sign">
              <strong>THE HALL OF BROTHERS</strong>
              <small>Build your own doors and preview the worlds beyond, below the scene.</small>
            </div>}

            {room.id === 'back-porch' && <div className="porch-sign">
              <span>Beyond this railing:</span>
              <strong>INFINITE COMMONS</strong>
              <small>Illustrative gateway · no shared session</small>
            </div>}
          </section>

          {room.id === 'hall-of-brothers' && <HallOfBrothers/>}

          <div className="content-grid">
            <section className="panel" aria-labelledby="objects-title">
              <p className="overline">LOOK AROUND</p>
              <h3 id="objects-title">Things in this room</h3>
              <div className="hotspots">
                {room.hotspots.map((spot) => (
                  <button key={spot.id} className="hotspot" onClick={() => inspect(spot.label, spot.text)}>
                    <span aria-hidden="true">✳</span> {spot.label} <span aria-hidden="true">↗</span>
                  </button>
                ))}
              </div>
              {detail ? <div className="discovery" role="status">
                <strong>{detail.title}</strong><p>{detail.text}</p>
                <button className="dismiss" onClick={() => setDetail(null)}>Close description</button>
              </div> : <p className="hint">Try an object or a Hall of Brothers door. All interactions are local, illustrative text.</p>}
            </section>

            <section className="panel" aria-labelledby="exits-title">
              <p className="overline">KEEP WANDERING</p>
              <h3 id="exits-title">Doorways</h3>
              <div className="exits">{room.exits.map((exit) => (
                <button key={exit} className="exit" onClick={() => go(exit)}>
                  <span aria-hidden="true">→</span> {rooms.find((item) => item.id === exit)?.name ?? exit}
                </button>
              ))}</div>
              {room.id === 'back-porch' &&
                <a className="commons-link" href={manifest.commons_preview_url} target="_blank" rel="noopener noreferrer">
                  Open Infinite Commons public preview ↗
                </a>
              }
              {room.id === 'back-porch' && <p className="hint">Opens a separate public site if deployed. It is not authenticated travel.</p>}
            </section>
          </div>
          <footer>LEDGER ABOVE EGO · PERSONAL ROOMS ARE PRIVATE BY DESIGN · THIS PREVIEW DOES NOT CONNECT TO YOUR NODE</footer>
        </main>
      </div>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(<House />);
