use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventListener;
use web_sys::HtmlElement;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.

    Ok(())
}


#[wasm_bindgen]
pub fn message(msg:&str){
    let prefix = msg.to_string()[0..3].to_string();

    //log(&format!("Message from {}", msg));

    match prefix.as_str() {
        "USR" => {
            // Takes in a message, splits it by the "|||||", and put every other one as a new paragraph
            // The <br> tag doesnt work for some reason
            let mut displayable_users = "".to_string();
            let mut newline_next = false;

            log("Current users are:");

            for i in msg.replace("USR ", "").split("||||||"){
                displayable_users.push_str(i);
                if newline_next {
                    log(&displayable_users);
                    displayable_users = "".to_string();
                    newline_next = false;
                } else {
                    displayable_users.push_str(" - ");
                    newline_next = true;
                }
            }
        },
        "VID" => {
            hide_video();
            let slug = &msg[4..];
            play_video(&format!("https://v.animethemes.moe/{}",slug));

            let display_only_checkbox = element_id("displayMode");
            let x = display_only_checkbox.value_of().as_bool().unwrap();

            

        }
        "DSP" => {
            if &msg[4..] == "true" {
                show_video();
            } else if &msg[4..] == "false" {
                hide_video();
            } else {
                log("Error: bad DSP command");
            }

        }
        "ANS" => {
            log(&format!("The correct answer is: {}", msg.replace("ANS ", "")));
        }
        "OAN" => {
            // Takes in a message, splits it by the "|||||", and put every other one as a new paragraph
            // The <br> tag doesnt work for some reason
            let mut displayable_users = "".to_string();
            let mut newline_next = false;

            log("Other answers were:");

            for i in msg.replace("OAN ", "").split("||||||"){
                displayable_users.push_str(i);
                if newline_next {
                    log(&displayable_users);
                    displayable_users = "".to_string();
                    newline_next = false;
                } else {
                    displayable_users.push_str(" - ");
                    newline_next = true;
                }
            }
        },
        _ => log(&format!("Unknown message: {}", msg)),
    }
}

// Plays a video with url "url" on the video player
#[wasm_bindgen]
pub fn play_video(url:&str){
    let player = element_id("videoplayer");
    expect_alert(player.set_attribute("src", url),"Can't set video");
    
}

// Un-hides the video player
#[wasm_bindgen]
pub fn show_video(){
    let player = element_id("videoplayer");
    expect_alert(player.remove_attribute("hidden"),"Can't hide video");
}

// Hides the video player
#[wasm_bindgen]
pub fn hide_video(){
    let player = element_id("videoplayer");
    expect_alert(player.set_attribute("hidden","true"),"Can't hide video");
}

#[wasm_bindgen]
pub fn log(msg:&str){
    let right_div = element_id("rightDiv");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let val = document.create_element("p").expect("could not create element");
    val.set_text_content(Some(msg));

    expect_alert(right_div.append_child(&val),"Can't add log element to right div");
}

fn expect_alert<E: std::fmt::Debug,T>(val:Result<T, E>, msg:&str) -> T{
    match val {
        Ok(success) => return success,
        Err(err) => {
            alert(&format!("Error:{} \n{:?}",msg,err));
            panic!("Error:{} \n{:?}",msg,err);
        },
    }
}

fn element_id(id:&str) -> HtmlElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let element = document.get_element_by_id(id)
		.expect(&format!("could not get element {}",id))
		.dyn_into::<web_sys::HtmlElement>()
		.expect(&format!("could not convert element {}",id));
    
    return element;
}
