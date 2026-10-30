//! Fixed-width source retirement records, compressed as one stream. X/Y are
//! retained only at generated indexed-extra PCs; other index witnesses are null.
use super::*;
mod generated { include!("../../zelda3/src/rom_block_costs.rs"); }
impl Inventory {
    pub fn ordered() -> Self {
        Self {extras:generated::BLOCKS.iter().flat_map(|b|b.6.iter().copied()).collect(),..Self::default()}
    }
    pub fn retire_instruction(&mut self,r:&crate::bridge::Instruction)->Result<()> {
        let width=u8::from(r.m8)|(u8::from(r.x8)<<1);
        let v=self.pending.entry((r.pc,width)).or_default();v[0]+=1;v[1]+=r.master;
        self.count+=1;self.clocks+=r.master;
        if self.count>2_000_000 {return Err("bounded ordered source iteration".into());}
        self.ordered.extend(r.pc.to_le_bytes());self.ordered.extend(r.retired_next_pc.ok_or("source retired target unavailable")?.to_le_bytes());
        self.ordered.extend(u32::try_from(r.master)?.to_le_bytes());self.ordered.extend(r.sp.to_le_bytes());self.ordered.extend(r.dp.to_le_bytes());
        let indices=self.extras.contains(&r.pc);
        self.ordered.extend(if indices{r.x.ok_or("indexed X witness unavailable")?}else{0}.to_le_bytes());
        self.ordered.extend(if indices{r.y.ok_or("indexed Y witness unavailable")?}else{0}.to_le_bytes());
        self.ordered.extend([r.db,r.opcode,width|(u8::from(r.emulation)<<2)|(u8::from(r.interrupt_after)<<3)|(u8::from(indices)<<4)]);
        Ok(())
    }
    pub fn verify_ordered_row(&mut self,index:u64,out:&mut impl Write)->Result<()> {
        if index!=self.verified_rows || self.ordered.len()as u64!=23*self.count
            || self.pending.values().map(|v|v[0]).sum::<u64>()!=self.count
            || self.pending.values().map(|v|v[1]).sum::<u64>()!=self.clocks {return Err("ordered source retirement conservation".into());}
        out.write_all(&index.to_le_bytes())?;out.write_all(&self.count.to_le_bytes())?;out.write_all(&self.clocks.to_le_bytes())?;out.write_all(&self.ordered)?;
        for(key,v)in self.pending.drain(){let total=self.totals.entry(key).or_default();total[0]+=v[0];total[1]+=v[1];}
        self.ordered.clear();self.count=0;self.clocks=0;self.verified_rows+=1;Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_roundtrip_retains_resume_and_extra_witnesses_and_discards_open_suffix() {
        let mut i=Inventory::ordered();let mut out=Vec::new();
        let mut r=crate::bridge::Instruction{iteration:0,ordinal:0,pc:0x088cac,opcode:0xb9,master:38,
            sp:0x1f6,dp:0,db:8,m8:true,x8:true,emulation:false,interrupt_after:true,
            retired_next_pc:Some(0x088caf),next_pc:0x8080,next_sp:0x1f2,x:Some(4),y:Some(0x41)};
        i.retire_instruction(&r).unwrap();r.pc=0x088caf;r.opcode=0x18;r.master=14;r.interrupt_after=false;r.retired_next_pc=Some(0x088cb0);
        i.retire_instruction(&r).unwrap();i.verify_ordered_row(0,&mut out).unwrap();i.retire_instruction(&r).unwrap();
        let mut input=std::io::Cursor::new(out);let (id,total,rows)=crate::bridge::route::source_row(&mut input).unwrap().unwrap();
        assert_eq!((id,total,rows.len()),(0,52,2));assert_eq!(rows[0].pc,0x088cac);assert_eq!(rows[0].y,Some(0x41));
        assert!(rows[0].interrupt_after);assert_eq!(rows[0].retired_next_pc,Some(0x088caf));assert_eq!(rows[0].next_sp,0x1f6);
        assert_eq!(rows[1].x,None);assert_eq!(rows[1].pc,0x088caf);assert!(crate::bridge::route::source_row(&mut input).unwrap().is_none());
    }
}
