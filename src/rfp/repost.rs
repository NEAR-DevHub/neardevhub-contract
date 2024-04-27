use crate::RFP;

pub fn rfp_repost_text(rfp: RFP) -> String {
    let rfp_link = format!("/devhub.near/widget/app?page=rfp&id={}", rfp.id);

    let title = rfp.snapshot.body.clone().latest_version().name;
    let summary = rfp.snapshot.body.clone().latest_version().summary;
    let category = rfp.snapshot.body.clone().latest_version().category;

    let text = format!(
        "Infrastructure Committee published a new *{category}* Request for Proposals.\n\n———\n\n**Title**: “{title}“\n\n**Summary**:\n\n{summary}\n\n———\n\nRead the full RFP and participate on [Infrastructure Committee page]({rfp_link})",
        rfp_link = rfp_link,
        title = title,
        summary = summary,
        category = category
    );

    text
}