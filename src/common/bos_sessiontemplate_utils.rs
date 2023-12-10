use mesa::shasta::ims;
use serde_json::Value;

pub async fn get_image_id_related_to_cfs_configuration(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name: &String,
) -> Option<String> {
    // Get all BOS sessiontemplates
    let bos_sessiontemplate_value_list = mesa::shasta::bos::template::http_client::get_all(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
    )
    .await
    .unwrap();

    get_image_id_from_bos_sessiontemplate_list(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        cfs_configuration_name,
        &bos_sessiontemplate_value_list,
    )
    .await
}

pub async fn get_image_id_from_bos_sessiontemplate_list(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_configuration_name: &String,
    bos_sessiontemplate_value_list: &[Value],
) -> Option<String> {
    // Get all BOS sessiontemplates related to CFS configuration
    let bos_sessiontemplate_value_target_list =
        bos_sessiontemplate_value_list
            .iter()
            .filter(|bos_session_template| {
                bos_session_template
                    .pointer("/cfs/configuration")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .eq(cfs_configuration_name)
            });

    for bos_sessiontemplate_value_target in bos_sessiontemplate_value_target_list {
        log::debug!(
            "BOS sessiontemplate details:\n{:#?}",
            bos_sessiontemplate_value_target
        );

        let bos_sessiontemplate_name = bos_sessiontemplate_value_target["name"].as_str().unwrap();

        for (_boot_sets_param, boot_sets_value) in bos_sessiontemplate_value_target["boot_sets"]
            .as_object()
            .unwrap()
        {
            if let Some(path) = boot_sets_value.get("path") {
                let image_id_related_to_bos_sessiontemplate = path
                    .as_str()
                    .unwrap()
                    .trim_start_matches("s3://boot-images/")
                    .trim_end_matches("/manifest.json")
                    .to_string();

                log::info!(
                    "Get image details for ID {}",
                    image_id_related_to_bos_sessiontemplate
                );

                if ims::image::http_client::get_raw(
                    shasta_token,
                    shasta_base_url,
                    shasta_root_cert,
                    &vec![],
                    Some(&image_id_related_to_bos_sessiontemplate),
                    None,
                    None,
                )
                .await
                .is_ok()
                {
                    log::info!(
                        "Image ID found related to BOS sessiontemplate {} is {}",
                        bos_sessiontemplate_name,
                        image_id_related_to_bos_sessiontemplate
                    );

                    return Some(image_id_related_to_bos_sessiontemplate);
                };
            }
        }
    }

    None
}
