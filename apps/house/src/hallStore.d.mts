export type HallTheme='amber'|'cobalt'|'moss'|'violet';
export interface HallDoor {
  id:string; companion:string; kingdom:string; description:string;
  theme:HallTheme; source:'example'|'owner-draft'; connection:'UNCONNECTED';
}
export interface HallManifest { schema:1; kind:'INFINITE_HOUSE_LOCAL_DRAFT'; doors:HallDoor[]; }
export interface HallLoad {hall:HallManifest;error:string;blocked:boolean;persisted:boolean;}
export const HALL_STORAGE_KEY:string;
export const HALL_KIND:'INFINITE_HOUSE_LOCAL_DRAFT';
export const HALL_THEMES:readonly HallTheme[];
export const HALL_MAX_DOORS:number;
export const HALL_MAX_BYTES:number;
export const HALL_LABEL_LIMITS:Readonly<{companion:number;kingdom:number;description:number}>;
export function defaultHall():HallManifest;
export function validateHall(input:unknown):HallManifest;
export function parseHall(input:string):HallManifest;
export function readHall(storage:Pick<Storage,'getItem'>):HallLoad;
export function writeHall(storage:Pick<Storage,'setItem'>,hall:HallManifest):HallManifest;
export function upsertDoor(hall:HallManifest, fields:{companion:string;kingdom:string;description:string;theme:HallTheme},id:string):HallManifest;
export function removeDoor(hall:HallManifest,id:string):HallManifest;
