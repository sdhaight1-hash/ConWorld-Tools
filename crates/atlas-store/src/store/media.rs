use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use std::io::Cursor;

const MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct MediaSummary {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub revision: i64,
}

/// Decode and reencode pixel data, intentionally discarding source metadata.
fn sanitize_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), Error> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(Error::InvalidInput("PNG asset size or signature"));
    }
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|_| Error::InvalidInput("PNG decode"))?;
    let info = reader.info();
    if info.width == 0 || info.height == 0 || info.width > 2048 || info.height > 2048 || info.animation_control.is_some() {
        return Err(Error::InvalidInput("PNG dimensions or animation"));
    }
    let count = reader.output_buffer_size().ok_or(Error::QueryBudgetExceeded)?;
    if count > 16 * 1024 * 1024 { return Err(Error::QueryBudgetExceeded); }
    let mut pixels = vec![0; count];
    let frame = reader.next_frame(&mut pixels).map_err(|_| Error::InvalidInput("PNG pixels"))?;
    let mut clean = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut clean, frame.width, frame.height);
        encoder.set_color(frame.color_type);
        encoder.set_depth(frame.bit_depth);
        let mut writer = encoder.write_header().map_err(|_| Error::InvalidInput("PNG encoding"))?;
        writer.write_image_data(&pixels[..frame.buffer_size()]).map_err(|_| Error::InvalidInput("PNG encoding"))?;
        writer.finish().map_err(|_| Error::InvalidInput("PNG encoding"))?;
    }
    if clean.len() > MAX_BYTES { return Err(Error::QueryBudgetExceeded); }
    Ok((frame.width, frame.height, clean))
}

impl Store {
    pub fn import_png(&mut self, view: ViewContext, name: &str, bytes: &[u8], command: Uuid) -> Result<MediaSummary, Error> {
        require_author(view.scope)?;
        let world = load_world(&self.conn, view.world)?;
        if world.main != view.branch { return Err(Error::CapabilityDenied); }
        if name.trim().is_empty() || name.len() > 200 { return Err(Error::InvalidInput("asset name")); }
        let (width, height, clean) = sanitize_png(bytes)?;
        let hash = Sha256::digest(&clean);
        let tx = self.conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let world = load_world(&tx, view.world)?;
        let revision = append_revision(&tx, &world, view.branch, view.revision, command, ("Local author", "Import sanitized PNG asset"))?;
        let id = Uuid::new_v4();
        tx.execute("INSERT INTO media_assets(world_id,id,name,width,height,sha256,bytes,revision) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![view.world.0.as_bytes(), id.as_bytes(), name.trim(), width, height, &hash[..], clean, revision])?;
        tx.commit()?;
        Ok(MediaSummary { id, name: name.trim().into(), width, height, revision })
    }

    pub fn media_assets(&self, view: ViewContext) -> Result<Vec<MediaSummary>, Error> {
        require_author(view.scope)?;
        let world = load_world(&self.conn, view.world)?;
        if world.main != view.branch { return Err(Error::CapabilityDenied); }
        if view.revision.0 < 0 || view.revision > world.revision { return Err(Error::InvalidRevision); }
        let mut stmt = self.conn.prepare("SELECT id,name,width,height,revision FROM media_assets WHERE world_id=?1 AND revision<=?2 ORDER BY name,id LIMIT 201")?;
        let mut rows = stmt.query(params![view.world.0.as_bytes(), view.revision.0])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            if result.len() == 200 { return Err(Error::QueryBudgetExceeded); }
            result.push(MediaSummary { id: Uuid::from_bytes(row.get(0)?), name: row.get(1)?, width: row.get(2)?, height: row.get(3)?, revision: row.get(4)? });
        }
        Ok(result)
    }

    // Reached only from the authorized layout projection.
    pub(crate) fn project_png(&self, view: ViewContext, id: Uuid) -> Result<(u32,u32,String,usize), Error> {
        let (width,height,hash,bytes):(u32,u32,Vec<u8>,Vec<u8>) = self.conn.query_row(
            "SELECT width,height,sha256,bytes FROM media_assets WHERE world_id=?1 AND id=?2 AND revision<=?3",
            params![view.world.0.as_bytes(),id.as_bytes(),view.revision.0],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)))
            .optional()?.ok_or(Error::NotFound)?;
        verify_bytes(&hash,&bytes)?;
        Ok((width,height,STANDARD.encode(&bytes),bytes.len()))
    }
}

fn verify_bytes(hash: &[u8], bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_BYTES || hash != &Sha256::digest(bytes)[..] {
        return Err(Error::Integrity("media SHA-256 mismatch".into()));
    }
    Ok(())
}

pub(crate) fn verify_media_hashes(conn: &Connection) -> Result<(), Error> {
    let mut stmt = conn.prepare("SELECT sha256,bytes FROM media_assets")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        verify_bytes(&row.get::<_,Vec<u8>>(0)?, &row.get::<_,Vec<u8>>(1)?)?;
    }
    Ok(())
}
