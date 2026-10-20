// Offline reference decoder. No executable code or visual/audio assets are imported.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
const bytes = readFileSync(process.argv[2] ?? '.reference/level1.bin');
if (bytes.length !== 2305) throw new Error('expected a 2305-byte DOS level record');
const glyphs = {0:' ',1:'.',2:'s',3:'o',4:'g',5:'.',6:'-',7:'.',8:'o',9:' ',10:'p',11:'l',12:' ',13:'.',14:'b',15:'+',16:'x',17:'e',18:'c',19:'*',20:'#',21:'k',22:'w',23:'.',24:'.',25:'o',26:' ',27:' ',28:' ',29:' ',30:'*'};
const rooms = Array.from({length:24}, (_,r) => {
  const tiles = Array.from({length:30},(_,i)=>bytes[r*30+i]&31);
  const switches = [];
  for (let i=0;i<30;i++) if ([6,15].includes(tiles[i])) {
    const targets=[];
    for(let n=bytes[720+r*30+i];n<256;n++) {
      const a=bytes[1440+n],b=bytes[1696+n];
      targets.push({room:((a&96)>>5)+((b&224)>>3),tile:a&31});
      if(a&128) break;
    }
    switches.push({tile:i,opens:tiles[i]===15,targets});
  }
  const guardTile=bytes[2119+r];
  return {id:r+1,links:Array.from(bytes.subarray(1952+r*4,1956+r*4)),
    rows:[0,1,2].map(y=>tiles.slice(y*10,y*10+10).map(t=>glyphs[t]??'.').join('')),
    switches,guard:guardTile<30?{tile:guardTile,facing:bytes[2143+r]===255?-1:1}:null};
});
const level={reference:'SDLPoP 3c5add5fb7f83d4ceb542823ab66d00146c4271b / res2001.bin',sha256:createHash('sha256').update(bytes).digest('hex'),start:{room:bytes[2112],tile:bytes[2113],facing:bytes[2114]===255?-1:1},rooms};
mkdirSync('assets',{recursive:true});
writeFileSync('assets/opening.json',JSON.stringify(level,null,2)+'\n');
console.log(`Decoded ${rooms.length} rooms; ${rooms.filter(r=>r.guard).length} guards; SHA256 ${level.sha256}`);
