import type { ReactNode } from 'react';
import './scene-art.css';

/**
 * Pure, original SVG illustration. Decorative only: a drawn device or person never
 * implies a connected Porch resource, a verified visitor, or an AI identity.
 * All interaction and permission boundaries remain outside this component.
 */
type SceneProps = { roomId: string; motionOn: boolean };

const palettes: Record<string, { wall: string; floor: string; trim: string }> = {
  bedroom: { wall: '#79574b', floor: '#53362c', trim: '#d3a077' },
  hallway: { wall: '#856044', floor: '#583a35', trim: '#cca77b' },
  'living-room': { wall: '#765e56', floor: '#563a3a', trim: '#d3a17b' },
  kitchen: { wall: '#9a866b', floor: '#594e42', trim: '#dfc9a1' },
  den: { wall: '#57433a', floor: '#3c2b2f', trim: '#cba573' },
  basement: { wall: '#372f4e', floor: '#29253a', trim: '#a882b7' },
  garage: { wall: '#6e6960', floor: '#52483d', trim: '#cbb78c' },
  attic: { wall: '#765a42', floor: '#48372f', trim: '#bc956c' },
  'hall-of-brothers': { wall: '#59405d', floor: '#3e304a', trim: '#deb987' },
  'back-porch': { wall: '#1b3153', floor: '#795641', trim: '#ecc08c' },
};

export const ILLUSTRATED_ROOMS = Object.keys(palettes);

const Rect = ({ x, y, width, height, fill, rx = 0 }: { x: number; y: number; width: number; height: number; fill: string; rx?: number }) =>
  <rect x={x} y={y} width={width} height={height} fill={fill} rx={rx} />;

function NightWindow({ x=85, y=70, width=280, height=270 }: {x?:number;y?:number;width?:number;height?:number}) {
  return <g>
    <Rect x={x-15} y={y-15} width={width+30} height={height+30} fill="#6d4b37"/>
    <Rect x={x} y={y} width={width} height={height} fill="url(#night)"/>
    <g className="house-stars"><circle cx={x+52} cy={y+51} r="2" fill="#fff2cb"/><circle cx={x+181} cy={y+37} r="3" fill="#fff2cb"/><circle cx={x+240} cy={y+112} r="2" fill="#fff2cb"/></g>
    <path d={`M${x} ${y+height} L${x+75} ${y+height-75} L${x+132} ${y+height-34} L${x+207} ${y+height-125} L${x+width} ${y+height-43} V${y+height} Z`} fill="#172a47"/>
    <path d={`M${x+width/2} ${y} v${height} M${x} ${y+height/2} h${width}`} stroke="#e4b480" strokeWidth="12"/>
    <Rect x={x-24} y={y+height} width={width+48} height={18} fill="#c79c76" rx={2}/>
    <g className="house-rain" opacity=".5" stroke="#c6e7ed" strokeWidth="2">
      <path d={`M${x+35} ${y+16} l-23 65 M${x+90} ${y+51} l-18 52 M${x+219} ${y+19} l-18 55 M${x+256} ${y+133} l-19 55`}/>
    </g>
  </g>;
}
function Computer({ x=470, y=320 }: {x?:number;y?:number}) {
  return <g>
    <rect x={x-24} y={y+119} width="255" height="24" fill="#b9895c"/>
    <rect x={x} y={y-4} width="164" height="135" rx="15" fill="#d9c5a5" stroke="#a39177" strokeWidth="9"/>
    <rect x={x+17} y={y+14} width="130" height="90" rx="12" fill="#0a302d"/>
    <text x={x+25} y={y+42} fontFamily="monospace" fontSize="14" fill="#75ecaa">INFINITE</text>
    <text x={x+25} y={y+63} fontFamily="monospace" fontSize="14" fill="#75ecaa">PORCH_</text>
    <rect className="house-crt" x={x+17} y={y+14} width="130" height="90" rx="12" fill="url(#scan)" opacity=".18"/>
    <circle cx={x+139} cy={y+117} r="3" fill="#7bcd82" className="house-led"/>
    <rect x={x-6} y={y+144} width="203" height="19" rx="6" fill="#e0cba9"/>
    <path d={`M${x+11} ${y+149} h164 M${x+25} ${y+154} h140`} stroke="#7b7260" strokeDasharray="6 5" strokeWidth="2"/>
  </g>;
}
function Lamp({x=830,y=160}: {x?:number;y?:number}) {
  return <g><path d={`M${x-38} ${y+75} L${x-22} ${y} H${x+22} L${x+38} ${y+75} Z`} fill="#f9c887"/>
    <ellipse className="house-lamp" cx={x} cy={y+85} rx="83" ry="50" fill="#ffd28d" opacity=".13"/>
    <path d={`M${x} ${y+75} v165 M${x-38} ${y+244} h76`} stroke="#d2a67c" strokeWidth="10"/></g>;
}
function Couch({ x=340, y=345 }: {x?:number;y?:number}) {
  return <g><rect x={x} y={y+52} width="430" height="143" rx="21" fill="#80515a" stroke="#ba8180" strokeWidth="10"/>
    <rect x={x+27} y={y} width="180" height="122" rx="35" fill="#c58d80"/><rect x={x+216} y={y} width="184" height="122" rx="35" fill="#c58d80"/>
    <path d={`M${x+216} ${y+15} v140`} stroke="#9a665f" strokeWidth="6"/><rect x={x-28} y={y+102} width="92" height="70" rx="19" fill="#a86966"/><rect x={x+370} y={y+102} width="92" height="70" rx="19" fill="#a86966"/>
    <path d={`M${x+30} ${y+196} v26 M${x+385} ${y+196} v26`} stroke="#a87a57" strokeWidth="14"/></g>;
}
function Television({x=750,y=270}: {x?:number;y?:number}) {
  return <g><rect x={x-18} y={y+180} width="310" height="86" rx="7" fill="#714a36"/>
    <rect x={x} y={y} width="270" height="210" rx="32" fill="#9e816d" stroke="#4f3d3a" strokeWidth="9"/>
    <rect x={x+22} y={y+18} width="210" height="160" rx="28" fill="#1c2745"/>
    <rect className="house-crt" x={x+22} y={y+18} width="210" height="160" rx="28" fill="url(#scan)" opacity=".24"/>
    <text x={x+127} y={y+84} textAnchor="middle" fontFamily="monospace" fontSize="19" fill="#a0dbef">CHANNEL</text>
    <text x={x+127} y={y+113} textAnchor="middle" fontFamily="monospace" fontSize="22" fill="#a0dbef">08</text>
    <circle cx={x+251} cy={y+70} r="9" fill="#d4c4a0"/><circle cx={x+251} cy={y+116} r="9" fill="#d4c4a0"/></g>;
}
function Bookcase({x=80,y=75,width=400}: {x?:number;y?:number;width?:number}) {
  return <g><rect x={x} y={y} width={width} height="430" fill="#503526" stroke="#ab744b" strokeWidth="15"/>
    {[0,1,2,3].map(row=><g key={row}><rect x={x+9} y={y+103+row*100} width={width-18} height="12" fill="#be8a5e"/>
      {Array.from({length:Math.floor((width-36)/29)},(_,i)=><rect key={i} x={x+18+i*29} y={y+12+row*100+(i%3)*9} width={18+i%3} height={87-(i%3)*9} rx="1" fill={['#b87960','#6e9c87','#ad8c64','#8c718f'][i%4]}/>)}</g>)}</g>;
}

function RoomObjects({ id }: {id:string}):ReactNode {
  switch(id) {
    case 'bedroom': return <g><NightWindow/><Lamp x={929} y={210}/><Computer x={405} y={272}/>
      <path d="M735 476 h297 v75 H735 Z" fill="#794c49"/><rect x="710" y="430" width="338" height="102" rx="20" fill="#ac7580"/><path d="M728 454 H1038" stroke="#ddaea6" strokeWidth="10"/><rect x="758" y="416" width="113" height="42" rx="14" fill="#e7c9ac"/>
      <rect x="560" y="154" width="120" height="104" fill="#3e343c" stroke="#ca9d74" strokeWidth="10"/><text x="620" y="204" textAnchor="middle" fontSize="19" fontFamily="serif" fill="#f2c993">1987</text><text x="620" y="226" textAnchor="middle" fontSize="13" fill="#f2c993">FRIDAY</text></g>;
    case 'hallway': return <g><path d="M0 0 H1100 L730 310 H360 Z" fill="#936a4e" opacity=".75"/><path d="M0 620 L370 310 H735 L1100 620 Z" fill="#46303d"/><path d="M355 620 L500 312 H620 L750 620 Z" fill="#96634d"/>
      {[{x:68,w:190},{x:830,w:200}].map(({x,w},i)=><g key={i}><rect x={x} y={135} width={w} height="385" fill="#48302e" stroke="#c69b73" strokeWidth="16"/><circle cx={x+w-35} cy="335" r="10" fill="#f4bf78"/></g>)}
      <rect x="449" y="126" width="208" height="248" fill="#53374a" stroke="#c59d7e" strokeWidth="12"/><text x="553" y="226" textAnchor="middle" fontSize="25" fill="#ffcb93">WELCOME</text><text x="553" y="259" textAnchor="middle" fontSize="19" fill="#ffcb93">HOME</text>
      <ellipse cx="545" cy="57" rx="50" ry="35" fill="#ffe0a0" opacity=".65"/><circle cx="545" cy="56" r="24" fill="#ffd99f"/><Lamp x={790} y={100}/></g>;
    case 'living-room': return <g><NightWindow x={75} y={65} width={260} height={250}/><Lamp x={380} y={149}/><Couch x={275} y={315}/><Television x={795} y={229}/>
      <rect x="405" y="536" width="290" height="40" rx="15" fill="#9e704d"/><rect x="420" y="570" width="12" height="46" fill="#6e493b"/>
      <rect x="415" y="167" width="136" height="112" stroke="#ba8d63" strokeWidth="11" fill="#394f55"/><path d="M433 255 L471 212 L505 238 L533 198" fill="none" stroke="#c6bd96" strokeWidth="8"/></g>;
    case 'kitchen': return <g><rect x="60" y="110" width="265" height="380" rx="12" fill="#decfae" stroke="#a09280" strokeWidth="11"/>
      <path d="M67 290 h250" stroke="#87776d" strokeWidth="7"/><circle cx="287" cy="380" r="10" fill="#95715c"/><circle cx="287" cy="212" r="10" fill="#95715c"/>
      <rect x="392" y="85" width="495" height="125" fill="#886849" stroke="#d0a976" strokeWidth="10"/><path d="M559 93 v110 M722 93 v110" stroke="#caa373" strokeWidth="9"/>
      <rect x="388" y="342" width="565" height="185" fill="#d0b58f" stroke="#947959" strokeWidth="9"/><path d="M388 418 h565 M563 343 v183 M736 343 v183" stroke="#a48b6b" strokeWidth="8"/>
      <rect x="363" y="311" width="615" height="26" fill="#b59a7b"/><ellipse cx="765" cy="308" rx="70" ry="15" fill="#6e7e80"/><path d="M765 294 v-37 q0 -24 36 -25" stroke="#bcbeb7" strokeWidth="10" fill="none"/>
      <rect x="540" y="213" width="125" height="95" fill="#f5eee0"/><text x="602" y="252" fontSize="20" textAnchor="middle" fill="#7b604c">COFFEE</text><text x="602" y="279" fontSize="18" textAnchor="middle" fill="#7b604c">TIME</text>
      <ellipse cx="554" cy="535" rx="245" ry="37" fill="#6b4f3e"/><path d="M510 550 v65 M607 550 v65" stroke="#4b312f" strokeWidth="18"/></g>;
    case 'den': return <g><Bookcase x={48} y={80} width={505}/><Bookcase x={750} y={80} width={300}/>
      <rect x="547" y="352" width="180" height="162" rx="55" fill="#728a70" stroke="#425c50" strokeWidth="12"/><rect x="537" y="475" width="205" height="58" rx="16" fill="#53735a"/>
      <Lamp x={690} y={169}/><rect x="559" y="545" width="170" height="30" fill="#a16e4c"/>
      <rect x="587" y="503" width="60" height="34" fill="#f0dbb6"/><text x="618" y="524" textAnchor="middle" fontSize="14" fill="#69503e">NBG</text></g>;
    case 'basement': return <g><rect x="93" y="70" width="830" height="495" fill="#29243e" stroke="#b5619d" strokeWidth="6"/>
      {[0,1,2].map(i=><g key={i} transform={`translate(${139+i*270} 0)`}>
        <path d="M0 112 h194 l-10 393 H10 Z" fill="#3b304e" stroke={['#f092ba','#71b6b6','#ffc17a'][i]} strokeWidth="9"/>
        <rect x="20" y="154" width="153" height="190" rx="20" fill="#152b43"/><rect className="house-crt" x="20" y="154" width="153" height="190" fill="url(#scan)" opacity=".3"/>
        <text x="96" y="214" textAnchor="middle" fontFamily="monospace" fontSize="17" fill="#f7d5be">{['PHICADE','SPARK','ARCADE'][i]}</text>
        <path d="M57 275 l38 -40 41 40 -41 39 Z" fill={['#f092ba','#71b6b6','#ffc17a'][i]} opacity=".8"/>
        <rect x="22" y="374" width="147" height="55" fill="#625369"/><circle cx="70" cy="402" r="12" fill="#d5aa85"/><circle cx="122" cy="402" r="12" fill="#ef718a"/>
      </g>)}<ellipse cx="565" cy="585" rx="450" ry="29" fill="#7c5483" opacity=".55"/></g>;
    case 'garage': return <g><rect x="45" y="65" width="1010" height="316" fill="#8b7458" stroke="#51493e" strokeWidth="15"/>
      {Array.from({length:12},(_,i)=>Array.from({length:4},(_,j)=><circle key={i+'-'+j} cx={97+i*79} cy={116+j*72} r="4" fill="#d3b994"/>))}
      <path d="M231 142 v130 M211 158 h55 M255 158 l55 73 M800 158 v175 M765 180 h65" fill="none" stroke="#d9cec0" strokeWidth="12"/>
      <rect x="95" y="420" width="900" height="31" fill="#cb9b65"/><rect x="130" y="447" width="74" height="160" fill="#886343"/><rect x="888" y="447" width="74" height="160" fill="#886343"/>
      <rect x="310" y="350" width="214" height="69" fill="#787668" stroke="#c6b59e" strokeWidth="12"/>
      <rect x="340" y="289" width="154" height="61" fill="#33474c"/><circle cx="418" cy="319" r="18" stroke="#d1b18b" fill="none" strokeWidth="9"/>
      <rect x="603" y="336" width="174" height="79" fill="#5a433e"/><rect x="645" y="297" width="90" height="42" fill="#96826b"/><path d="M675 302 v33" stroke="#d9bd8c" strokeWidth="11"/></g>;
    case 'attic': return <g><path d="M0 250 L555 -30 L1100 250" stroke="#b68659" strokeWidth="45" fill="none"/><path d="M170 187 l0 360 M900 184 v363 M170 185 l730 0" stroke="#795036" strokeWidth="20"/>
      <rect x="92" y="410" width="310" height="145" fill="#ad8564" stroke="#654837" strokeWidth="12"/><path d="M114 416 v140 M366 416 v140" stroke="#79543a" strokeWidth="11"/><text x="245" y="480" textAnchor="middle" fill="#e3cfb2" fontFamily="serif" fontSize="25">MEMORIES</text>
      <rect x="560" y="430" width="305" height="132" fill="#6b473a" stroke="#ba9060" strokeWidth="15" rx="8"/><path d="M573 475 h280 M700 436 v120" stroke="#cfac76" strokeWidth="13"/><circle cx="714" cy="472" r="15" fill="#e1c09b"/>
      <rect x="690" y="335" width="130" height="90" fill="#b38b6c" stroke="#876045" strokeWidth="10"/><path d="M695 380 h120" stroke="#7c5639" strokeWidth="8"/>
      <path d="M549 40 L734 386 L885 455 L705 51 Z" fill="#fce3a6" opacity=".12" className="house-dust"/></g>;
    case 'hall-of-brothers': return <g><path d="M20 0 L410 280 L440 620 H0 Z" fill="#533652"/><path d="M1080 0 L690 280 L660 620 H1100 Z" fill="#533652"/>
      <path d="M260 620 L480 278 H620 L845 620 Z" fill="#60465e"/>
      {[0,1,2,3].map(i=><g key={i} opacity={1-i*.17}><path d={`M${225+i*76} ${68+i*55} h${145-i*12} v${430-i*46} h-${145-i*12} Z`} fill="#33283e" stroke="#d1a675" strokeWidth={10-i*2}/><circle cx={345+i*70} cy={340+i*28} r="8" fill="#ffdf9b"/></g>)}
      <path d="M130 620 L460 290 M960 620 L640 290" stroke="#d7aa78" strokeWidth="9"/><text x="550" y="91" textAnchor="middle" fontSize="35" fill="#ffe2b0">HALL OF BROTHERS</text>
      <circle cx="554" cy="197" r="42" fill="#f6c47f" opacity=".32"/><path d="M548 145 v80 M515 185 h80" stroke="#ffdeb3" strokeWidth="6"/></g>;
    case 'back-porch': return <g>
      <rect x="0" y="0" width="1100" height="500" fill="url(#night)"/>
      <circle cx="870" cy="120" r="46" fill="#f2d9ad" opacity=".85"/><path d="M0 484 L180 322 L320 399 L528 196 L743 405 L933 296 L1100 455 V620 H0 Z" fill="#1d3851"/>
      <path d="M275 413 L526 196 L655 322 L542 295 L519 313 L493 281 Z" fill="#d9e5e1" opacity=".83"/>
      <path d="M0 510 L209 424 L350 470 L600 400 L780 483 L966 410 L1100 477 V620 H0 Z" fill="#173d47"/>
      <path d="M748 460 v-125 l40-72 40 72 v125 Z" fill="#6e9ba0" stroke="#a2d7dc" strokeWidth="6" opacity=".93"/>
      <path className="house-flame" d="M784 260 C759 230 770 196 791 176 C797 205 824 225 794 260 Z" fill="#76d8ff"/>
      <rect x="0" y="485" width="1100" height="135" fill="#8b5d41"/>
      {Array.from({length:13},(_,i)=><path key={i} d={`M${i*92} 500 v120`} stroke="#57382d" strokeWidth="5"/>)}
      <path d="M0 432 H1100 M60 431 v90 M260 431 v90 M460 431 v90 M660 431 v90 M860 431 v90 M1060 431 v90" stroke="#d2a274" strokeWidth="22"/>
      <path d="M210 555 q40 -80 90 0 h120 M218 555 l-29 40 M389 555 l26 40" stroke="#d5a97b" strokeWidth="14" fill="none"/>
      <path d="M0 0 h47 v480 M1053 0 v480" stroke="#593b36" strokeWidth="85"/>
      <text x="565" y="93" textAnchor="middle" fontFamily="serif" fontSize="30" fill="#fff1ca">Beyond the porch, the Commons...</text></g>;
    default: return null;
  }
}

export function RoomScene({roomId,motionOn}:SceneProps) {
 const palette=palettes[roomId] ?? palettes.bedroom;
 return <svg className="house-scene" viewBox="0 0 1100 620" preserveAspectRatio="xMidYMid slice" role="img" aria-label={`Original decorative illustration of the ${roomId.replaceAll('-', ' ')}. Objects are sample visuals, not live devices.`} data-motion={motionOn?'on':'off'} focusable="false">
  <defs>
   <linearGradient id="night" x1="0" x2="0" y1="0" y2="1"><stop offset="0%" stopColor="#131b3b"/><stop offset="70%" stopColor="#376a90"/><stop offset="100%" stopColor="#d99c78"/></linearGradient>
   <pattern id="wall-lines" width="90" height="90" patternUnits="userSpaceOnUse"><rect x="76" width="5" height="90" fill="#2b2030" opacity=".22"/></pattern>
   <pattern id="scan" width="3" height="7" patternUnits="userSpaceOnUse"><path d="M0 0 h3" stroke="#bcffcf" opacity=".5" strokeWidth="1"/></pattern>
  </defs>
  <rect width="1100" height="620" fill={palette.wall}/>
  <rect width="1100" height="445" fill="url(#wall-lines)"/>
  <rect y="445" width="1100" height="175" fill={palette.floor}/>
  {Array.from({length:14},(_,i)=><path key={i} d={`M${i*82} 445 l-19 175`} stroke="#231d25" strokeWidth="4" opacity=".22"/>)}
  <rect y="430" width="1100" height="23" fill={palette.trim}/>
  <RoomObjects id={roomId}/>
  <rect width="1100" height="620" fill="#20152a" opacity=".1" pointerEvents="none"/>
 </svg>;
}
