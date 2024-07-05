use crate::artwork::*;
use crate::math::*;
use crate::LoadedImage;
extern crate nalgebra as na;
use na::dvector;

#[derive(PartialEq, Copy, Clone)]
pub enum ReportStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ReportTypes {
    Dpi,
    PartialTransparency,
    Bib,
    AreaUsed,
}

pub struct ReportTemplate {
    pub label: String,
    pub report_tip: String,
    pub tool_tip: String,
    pub display_percent: bool,
    pub metric_to_status: fn(metric: u32) -> ReportStatus,
    pub generate_metric: fn(art: &LoadedImage, art_dependent_data: &ArtworkDependentData) -> u32,
}

fn dpi_to_status(dpi: u32) -> ReportStatus {
    match dpi {
        0..=199 => ReportStatus::Fail,
        200..=299 => ReportStatus::Warn,
        _ => ReportStatus::Pass,
    }
}

fn compute_dpi(art: &LoadedImage, _art_dependent_data: &ArtworkDependentData) -> u32 {
    let top_corner = art_to_art_space(art.size()) * dvector![0.0, 0.0, 1.0];
    let bot_corner = art_to_art_space(art.size()) * dvector![1.0, 1.0, 1.0];
    let dim_in_inches = bot_corner - top_corner;
    (art.size().x / dim_in_inches.x) as u32
}

fn bad_transparency_to_status(bad_transparency_pixels: u32) -> ReportStatus {
    match bad_transparency_pixels {
        0 => ReportStatus::Pass,
        _ => ReportStatus::Fail,
    }
}

fn area_used_to_status(area_used: u32) -> ReportStatus {
    match area_used {
        0..=50 => ReportStatus::Fail,
        51..=90 => ReportStatus::Warn,
        _ => ReportStatus::Pass,
    }
}

fn opaque_to_status(opaque_area: u32) -> ReportStatus {
    match opaque_area {
        0..=49 => ReportStatus::Pass,
        50..=74 => ReportStatus::Warn,
        _ => ReportStatus::Fail,
    }
}

fn compute_area_used(art: &LoadedImage, _art_dependent_data: &ArtworkDependentData) -> u32 {
    let top_corner = art_to_art_space(art.size()) * dvector![0.0, 0.0, 1.0];
    let bot_corner = art_to_art_space(art.size()) * dvector![1.0, 1.0, 1.0];
    let dim_in_inches = bot_corner - top_corner;
    let area_used = 100.0 * dim_in_inches[0] * dim_in_inches[1] / (11.0 * 14.0);
    area_used as u32
}

fn compute_bib_score(art: &LoadedImage, art_dependent_data: &ArtworkDependentData) -> u32 {
    let area_used = compute_area_used(art, art_dependent_data);
    area_used * art_dependent_data.opaque_percent / 100
}

fn compute_badtransparency_pixels(
    _art: &LoadedImage,
    art_dependent_data: &ArtworkDependentData,
) -> u32 {
    art_dependent_data.partial_transparency_percent
}

pub struct ReportTemplates {
    area_used_report: ReportTemplate,
    transparency_report: ReportTemplate,
    opaque_report: ReportTemplate,
    dpi_report: ReportTemplate,
}

impl ReportTemplates {
    pub fn new() -> Self {
        let dpi_report = ReportTemplate {
            label: "DPI".to_string(),
            report_tip: "Ideally, artwork for T-Shirts should be Print Quality - 300 DPI or more. Medium Quality (200 to 299 DPI) is probably okay. Below 200 DPI pixalation may be noticable.".to_string(),
            tool_tip: "Show close ups of areas where artwork might look pixelly.\nTurn off the tool or move the T-Shirt to exit.".to_string(),
            display_percent: false,
            metric_to_status: dpi_to_status,
            generate_metric: compute_dpi,
        };
        let area_used_report = ReportTemplate {
            label: "Area Used".to_string(),
            report_tip: "Artwork is usually printed on an 11 inch by 14 inch area of the T-Shirt.  The report shows how much of that printable area the art is currently filling.  There's no rule that says art has to use all of the available area, but it's nice to know how much available area there is.".to_string(),
            tool_tip: "Show the maximum boundary of the printable area on the T-Shirt.".to_string(),
            display_percent: true,
            metric_to_status: area_used_to_status,
            generate_metric: compute_area_used,
        };
        let transparency_report = ReportTemplate {
            label: "Partial\nTransparency".to_string(),
            report_tip: "The processed used to print T-Shirt artwork doesn't support partial transparency.  Either the artwork is being printed (100% transparecy) or the T-Shirt is showing through (0% transparency) - there's nothing in between.  For best results, fix partial transparency problems in your art package of choice.".to_string(),
            tool_tip: "Show areas of the artwork where there's partial transparency of some kind.".to_string(),
            display_percent: true,
            metric_to_status: bad_transparency_to_status,
            generate_metric: compute_badtransparency_pixels,
        };
        let opaque_report = ReportTemplate {
            label: "Bib Score".to_string(),
            report_tip: "T-Shirt artwork shouldn't cover all the printable area.  The more area the artwork covers, the more the T-Shirt will feel like a pastic bib you'd put on a baby for meal time.  For best results the artwork have transparent areas where the T-Shirt will show through, and work with the T-Shirt color.".to_string(),
            tool_tip: "TODO: have tool do something.".to_string(),
            display_percent: true,
            metric_to_status: opaque_to_status,
            generate_metric: compute_bib_score,
        };
        Self {
            area_used_report,
            dpi_report,
            opaque_report,
            transparency_report,
        }
    }

    pub fn report_type_to_template(&self, report_type: ReportTypes) -> &ReportTemplate {
        match report_type {
            ReportTypes::Dpi => &self.dpi_report,
            ReportTypes::AreaUsed => &self.area_used_report,
            ReportTypes::PartialTransparency => &self.transparency_report,
            ReportTypes::Bib => &self.opaque_report,
        }
    }
}
