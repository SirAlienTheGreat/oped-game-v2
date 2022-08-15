use soup::prelude::*;
use std::{fs, io::Read};
//use std::fmt::Error;
use std::io::{Error, ErrorKind};
use animethemes_rs::client::AnimeThemesClient;
use animethemes_rs::includes::SearchIncludes;
use rand::{seq::IteratorRandom, thread_rng};
use std::time::Duration;
use async_recursion::async_recursion;


// Gets a new theme
// If theres an error, it tries again
#[async_recursion]
pub async fn get_new_theme_recursive(list_of_anime:&Vec<String>) -> (String, String) {
    let potential_theme = get_new_theme(list_of_anime).await;
    return match potential_theme {
        Ok(theme) => theme,
        Err(error) => {
            println!("====== Error getting anime: {}", error);
            tokio::time::sleep(Duration::from_millis(1100)).await;
            get_new_theme_recursive(list_of_anime).await
        }
    }
}

async fn get_new_theme(list_of_anime:&Vec<String>) -> Result<(String, String), Error> {

    //let mut rng = thread_rng();

    let client = AnimeThemesClient::default();

    let mut anime_title;// = list_of_anime[0].clone();//.iter().choose(&mut rng).unwrap();

    {
        let mut rng = thread_rng();
        anime_title = list_of_anime.iter().choose(&mut rng).unwrap().to_string();
    }

    println!("asking animethemes.moe");
    let response = match match tokio::time::timeout(Duration::from_millis(4500),client.search( & anime_title,
        &[],
        SearchIncludes::all())).await {
            Ok(data) => data,
            Err(_) => return Result::Err(Error::new(ErrorKind::Other, "timeout")),
        } {
            Ok(data) => data,
            Err(error) => return Result::Err(Error::new(ErrorKind::Other, error)),
        };
    println!("asked animethemes.moe");

    let mut rng2 = thread_rng();

    let x = response.anime.unwrap();
    let bigg_slug = &x[0].themes.as_ref().unwrap()
        .iter().choose(&mut rng2).unwrap()
        .entries.as_ref().unwrap()[0]
        .videos.as_ref().unwrap()[0].path;
    // bigg_slug is a "slug", like "2012/Summer/YuruYuriS2-OP1.webm"
    
    let slug = bigg_slug.split("/").collect::<Vec<&str>>()[2].to_string();

    println!("slug is {}", slug);
    println!("Bigg slug is {:?}", bigg_slug);

    Ok((slug.to_string(), anime_title.to_string()))
}

pub fn get_all_anime() -> Vec<String>{
    println!("\nEnter path to folder comtaining MAL lists.\n
              All lists must end in -list.xml");
    
    let mut input = "/home/sir-alien-the-great/Downloads/newest-list".to_string();
    //io::stdin().read_line(&mut input).expect("A problem occurred");

    let paths = fs::read_dir(input.trim()).unwrap();
    let mut str_path :Vec<String> = vec![];

    for path in paths { // Convert path (:ReadDir) to :String
        if !path.as_ref().unwrap().path().display().to_string().contains("index"){
            str_path.push(path.unwrap().path().display().to_string());
        }

    }

    

    //let mut list_paths:Vec<String> = vec![];
    let mut str_lists:Vec<Vec<String>> = vec![];
    for i in &str_path {
        if i.contains("-list.xml") {
            //list_paths.push(i.to_string());
            
            let mut file = fs::File::open(i).expect("can't open file");
            let mut data = "".to_string();

            file.read_to_string(&mut data).expect("can't read contents of file");
            
            let list_of_names = get_names_from_list(&data);

            println!("len {} path: {}, ",list_of_names.len(),i);

            str_lists.push(list_of_names);

            

            // In the future, maybe make this multithreaded?

        }
    }

    // Find shortest list
    let mut shortest_list = str_lists[0].clone();
    for i in &str_lists {
        if i.len() < shortest_list.len() {
            shortest_list = i.to_vec();
        }
    }

    println!("shortest list: {}",shortest_list.len());

    // go through each anime in the shortest list and see if its in all the other lists
    let mut shared_list = vec![];
    for i in &shortest_list {
        let mut exists_in_all_lists = true;
        for j in &str_lists {
            if !j.contains(i) {
            //if !binary_search(j.to_vec(), i.to_string()) {
                exists_in_all_lists = false;
                println!("{} doesn't exist in list with len {}",i.to_string(), j.len());
            }
        }
        if exists_in_all_lists {
            shared_list.push(i.to_string());
        } else {
            println!("doesnt exist in all lists {}", i.to_string());
        }
    }

    //println!("{}", shared_list.join("\n"));

    return shared_list
    
}


fn get_names_from_list(list:&str) -> Vec<String> {
    let soup = Soup::new(list);

    let name = soup.tag("series_title").find_all();
    let status = soup.tag("my_status").find_all();
    
    let mut names:Vec<String> = vec![];
    let mut statuses:Vec<String> = vec![];

    for i in name {
        let x = i.display().replace("<series_title><!--[CDATA[", "").replace("]]--></series_title>", "");
        names.push(x);
    }

    for i in status {
        let x = i.display().replace("<my_status>", "").replace("</my_status>", "");
        statuses.push(x);
    }

    let mut output:Vec<String> = vec![];

    for i in 0..names.len() {
        if statuses[i] == "Completed" || statuses[i] == "Watching" {
            output.push(names[i].clone());
        }
    }

    return output;
}


// this is broken for a few conditions, but i dont have time to figure out why, so im going with the O(n) algorithm
fn binary_search(list:Vec<String>, query:String) -> bool {
    let index = list.len() as i32 /2 as i32;
    if list[index as usize] == query {
        return true;
    } else if list.len() == 1 {
        return false;
    } else if list[index as usize] > query {
        return binary_search(list[..index as usize].into(), query);
    } else if list[index as usize] < query {
        return binary_search(list[index as usize..].into(), query);
    } else {
        panic!("this shouldn't happen");
    }    
}