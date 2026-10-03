export type Privacy = 'LOCAL_ONLY' | 'TRUSTED_PEERS' | 'PORCH_ALLOWED' | 'FEDERATED_ALLOWED';
export type Signed<T> = {version:1;domain:string;signer:string;public_key:string;payload:T;signature:string};
export type RunRequest = {resource:string;input:string;privacy?:Privacy;capability?:'model.inference'|'compute.hash';max_output_tokens?:number;timeout_ms?:number;preferred_peer?:string;id?:string};
export type Receipt = {version:1;job_id:string;requester:string;executor:string;capability:string;resource:string;provider:string;model_version:string|null;input_digest:string;output_digest:string;request_digest:string;status:'COMPLETED'|'REFUSED';reason:string|null;output:unknown;started_at:number;finished_at:number;duration_ms:number;grant_nonce:string|null;measured:Record<string,unknown>};
export type RunResult = {status:'COMPLETED'|'REFUSED';reason?:string;route:Record<string,unknown>;receipt?:Signed<Receipt>};
/** Observations, proposals and local credential possession do not create peer grants. */
export class PorchClient {
  constructor(private token:string,private base='') {
    if(base) {const url=new URL(base);if(url.protocol!=='http:' || !['127.0.0.1','[::1]'].includes(url.hostname))throw new Error('Porch control API must be loopback');}
  }
  private async request<T>(path:string,body?:unknown):Promise<T>{
    const response=await fetch(`${this.base}${path}`,{method:body===undefined?'GET':'POST',headers:{Authorization:`Bearer ${this.token}`,...(body!==undefined?{'Content-Type':'application/json'}:{})},...(body!==undefined?{body:JSON.stringify(body)}:{}),redirect:'error',cache:'no-store'});
    const result=await response.json();if(!response.ok)throw new Error(result.reason || `API refused: ${response.status}`);return result;
  }
  read<T=unknown>(resource:string):Promise<T>{if(!/^[a-z]+$/.test(resource))throw new Error('Invalid API resource');return this.request<T>(`/v1/${resource}`);}
  control<T=unknown>(operation:string,args:Record<string,unknown>={}):Promise<T>{return this.request<T>('/v1/control',{operation,args});}
  run(request:RunRequest):Promise<RunResult>{return this.control('job.run',{...request,privacy:request.privacy??'LOCAL_ONLY'});}
  observeModels():Promise<unknown>{return this.read('models');}
  async stream(onEvent:(event:unknown)=>void,signal:AbortSignal):Promise<void>{
    const r=await fetch(`${this.base}/v1/events`,{headers:{Authorization:`Bearer ${this.token}`},signal,redirect:'error'});if(!r.ok || !r.body)throw new Error('Event stream refused');
    const reader=r.body.getReader(),decoder=new TextDecoder();let buffer='';
    while(!signal.aborted){const {value,done}=await reader.read();if(done)break;buffer+=decoder.decode(value,{stream:true});let index:number;
      while((index=buffer.indexOf('\n\n'))>=0){const packet=buffer.slice(0,index);buffer=buffer.slice(index+2);const data=packet.split('\n').filter(l=>l.startsWith('data:')).map(l=>l.slice(5).trim()).join('\n');if(data)onEvent(JSON.parse(data));}
      if(buffer.length>1048576)throw new Error('Event stream size limit');
    }
  }
}
export class PhiVesselPorchAdapter {constructor(private client:PorchClient){}observe(){return this.client.observeModels();}propose(request:RunRequest){return {request:{...request,privacy:request.privacy??'LOCAL_ONLY'},execution_authority:false as const};}execute(request:RunRequest){return this.client.run(request);}}
export class PhiBotPorchAdapter {constructor(private client:PorchClient){}execute(request:RunRequest){return this.client.run(request);}}
export class CommonLinePorchAdapter {constructor(private client:PorchClient){}observePeers(){return this.client.read('peers');}send(peer:string,text:string){return this.client.control('message.send',{peer,text});}}
