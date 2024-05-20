use crate::RFP;

pub fn rfp_repost_text(rfp: RFP) -> String {
    let rfp_link = format!("/devhub.near/widget/app?page=rfp&id={}", rfp.id);

    let body = rfp.snapshot.body.latest_version();

    let title = body.name;
    let summary = body.summary;

    let text = format!(
        "A new Request for Proposals is published.\n\n———\n\n**Title**: “{title}“\n\n**Summary**:\n\n{summary}\n\n———\n\nRead the full RFP and participate [here]({rfp_link})",
        rfp_link = rfp_link,
        title = title,
        summary = summary,
    );

    text
}
