#![allow(dead_code)]
#![allow(unused_variables)]

use std::io;
use std::fs::File;
use std::convert::Into;
use std::path::{PathBuf, Path};
use std::process::{Command, Output};

macro_rules! wrapper_builder {
    (
        name=$name:ident,
        commands={
            $($func_name:ident($cmd:tt $(,)? $($v:ident),*) ),*
        }
    ) => {
        pub struct $name {
            pub command: Command,
        }

        impl $name {
            pub fn new(execute: &Path) -> Self {
                $name {
                    command: Command::new(execute)
                }
            }

            pub fn execute_command(&mut self) -> io::Result<Output> {
                self.command.output()
            }

            $(
                pub fn $func_name<'a>(&'a mut self, $($v: &'a str),*) -> &'a mut $name {
                    self.command.arg($cmd);
                    $( self.command.arg($v); )*
                    self
                }
            )*
        }
    }
}

wrapper_builder!(
    name=YoutubeDlWrapper,
    commands = {
        help("--help"),
        version("--version"),
        update("--update"),
        ignore_errors("--ignore-errors"),
        abort_on_error("--abort-on-error"),
        dump_user_agent("--dump-user-agent"),
        list_extractors("--list-extractors"),
        extractor_descriptions("--extractor-descriptions"),
        force_generic_extractor("--force-generic-extractor"),
        default_search("--default-search", prefix),
        ignore_config("--ignore-config"),
        config_location("--config-location", path),
        flat_playlist("--flat-playlist"),
        mark_watched("--mark-watched"),
        no_mark_watched("--no-mark-watched"),
        no_color("--no-color"),
        // Network Options
        proxy("--proxy", url),
        socket_timeout("--socket-timeout", seconds),
        source_address("--source-address", ip),
        force_ipv4("--force-ipv4"),
        force_ipv6("--force-ipv6"),
        // Geo Restriction:
        geo_verification_proxy("--geo-verification-proxy", url),
        geo_bypass("--geo-bypass"),
        no_geo_bypass("--no-geo-bypass"),
        geo_bypass_county("--geo-bypass-country", code),
        geo_bypass_ip_block("--geo-bypass-ip-block", ip_block),
        // Video Selection
        playlist_start("--playlist-start", number),
        playlist_end("--playlist-end", number),
        playlist_items("--playlist-items", item_spec),
        match_title("--match-title", regex),
        reject_title("--reject-title", regex),
        max_downloads("--max-downloads", number),
        min_filesize("--min-filesize", size),
        max_filesize("--max-filesize", size),
        date("--date", date),
        datebefore("--datebefore", date),
        dateafter("--dateafter", date),
        min_views("--min-views", count),
        max_views("--max-views", count),
        match_filter("--match-filter", filter),
        no_playlist("--no-playlist"),
        yes_playlist("--yes-playlist"),
        age_limit("--age-limit", years),
        download_archive("--download-archive", file),
        include_ads("--include-ads"),
        // Download Options
        limit_rate("--limit-rate", rate),
        retries("--retries", retries),
        fragment_retries("--fragment-retries", retries),
        skip_unavailable_fragments("--skip-unavailable-fragments"),
        abort_on_unavailable_fragment("--abort-on-unavailable-fragment"),
        keep_fragments("--keep-fragments"),
        buffer_size("--buffer-size", size),
        no_resize_buffer("--no-resize-buffer"),
        http_chunk_size("--http-chunk-size", size),
        playlist_reverse("--playlist-reverse"),
        playlist_random("--playlist-random"),
        xattr_set_filesize("--xattr-set-filesize"),
        hls_prefer_native("--hls-prefer-native"),
        hls_prefer_ffmpeg("--hls-prefer-ffmpeg"),
        hls_use_mpegts("--hls-use-mpegts"),
        external_downloader("--external-downloader", command),
        external_downloader_args("--external-downloader-args", args),
        // Filesystem Options
        batch_file("--batch-file", file),
        id("--id"),
        output("--output", template),
        output_na_placeholder("--output-na-placeholder", placeholder),
        autonumber_start("--autonumber-start", number),
        restrict_filenames("--restrict-filenames"),
        no_overwrites("--no-overwrites"),
        force_continue("--continue"),
        no_continue("--no-continue"),
        no_part("--no-part"),
        no_mtime("--no-mtime"),
        write_description("--write-description"),
        write_info_json("--write-info-json"),
        write_annotations("--write-annotations"),
        load_info_json("--load-info-json", file),
        cookies("--cookies", file),
        cache_dir("--cache_dir", dir),
        no_cache_dir("--no-cache-dir"),
        rm_cache_dir("--rm_cache_dir"),
        // Thumbnail Options
        write_thumbnail("--write-thumbnail"),
        write_all_thumbnails("--write-all-thumbnails"),
        list_thumbnails("--list-thumbnails"),
        // Verbosity / Simulation Options
        quiet("--quiet"),
        no_warnings("--no-warnings"),
        simulate("--simulate"),
        skip_download("--skip-download"),
        get_url("--get-url"),
        get_title("--get-title"),
        get_id("--get-id"),
        get_thumbnail("--get-thumbnail"),
        get_description("--get-description"),
        get_duration("--get-duration"),
        get_filename("--get_filename"),
        get_format("--get-format"),
        dump_json("--dump-json"),
        dump_single_json("--dump-single-json"),
        print_json("--print-json"),
        newline("--newline"),
        no_progress("--no-progress"),
        console_title("--console-title"),
        verbose("--verbose"),
        dump_pages("--dump-pages"),
        write_pages("--write-pages"),
        print_traffic("--print-traffic"),
        call_home("--call-home"),
        no_call_home("--no-call-home"),
        // Workarounds
        encoding("--encoding", encoding),
        no_check_certificate("--no-check-certificate"),
        prefer_insecure("--prefer-insecure"),
        user_agent("--user-agent", ua),
        referer("--referer", url),
        add_header("--add-header", pattern),
        bidi_workaround("--bidi-workaround"),
        sleep_interval("--sleep-interval", seconds),
        max_sleep_interval("--max-sleep-interval"),
        // Video Format Options
        format("--format", format),
        all_formats("--all-formats"),
        prefer_free_formats("--prefer-free-formats"),
        list_formats("--list-formats"),
        youtube_skip_dash_manifest("--youtube-skip-dash-manifest"),
        merge_output_format("--merge-output-format", format),
        // Subtitle Options
        write_sub("--write-sub"),
        write_auto_sub("--write-auto-sub"),
        all_subs("--all-subs"),
        list_subs("--list-subs"),
        sub_format("--sub-format", format),
        sub_lang("--sub-lang", langs),
        // Authentication Options
        username("--username", username),
        password("--password", password),
        twofactor("--twofactor", auth_code),
        netrc("--netrc"),
        video_password("--video-password", password),
        // Adobe Pass Options
        ap_mso("--ap-mso", mso),
        ap_username("--ap-username", username),
        ap_password("--ap-password", password),
        ap_list_mso("--ap-list-mso"),
        // Post-processing Options
        extract_audio("--extract-audio"),
        audio_format("--audio-format", format),
        audio_quality("--audio-quality", quality),
        recode_video("--recode-video", format),
        postprocessor_args("--postprocessor-args", args),
        keep_video("--keep-video"),
        no_post_overwrites("--no-post-overwrites"),
        embed_thumbnail("--embed-thumbnail"),
        add_metadata("--add-metadata"),
        metadata_from_title("--metadata-from-title", format),
        xattrs("--xattrs"),
        fixup("--fixup", policy),
        prefer_avconv("--prefer-avconv"),
        prefer_ffmpeg("--prefer-ffmpeg"),
        ffmpeg_location("--prefer-ffmpeg", path),
        exec("--exec", cmd),
        convert_subs("--convert-subs", format)
    }
);