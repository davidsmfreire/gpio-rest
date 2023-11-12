use actix_web::{web::{Data, self}, get, Responder, HttpResponse, HttpServer, App};
use gpio::{GpioIn, GpioValue, sysfs::{SysFsGpioInput, SysFsGpioOutput}};
use serde_json::json;
use std::{fs::File, io::BufReader, error::Error, path::Path, collections::HashMap, sync::{Mutex, Arc}};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

type GpioInputPins = HashMap<u16, SysFsGpioInput>;
type GpioOutputPins = HashMap<u16, SysFsGpioOutput>;

struct GpioHandlers {
    inputs: GpioInputPins,
    _outputs: GpioOutputPins,
}

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
enum GpioMode {
    INPUT,
    OUTPUT
}

#[derive(Deserialize, Debug)]
struct GpioPinConfig {
    number: u16,
    mode: GpioMode
}

#[derive(Deserialize, Debug)]
struct GpioConfig {
    pins: Vec<GpioPinConfig>,
}

#[derive(Deserialize)]
struct PinRequest {
    id: u16,
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<GpioConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn prepare_gpio(config: &GpioConfig) -> Result<GpioHandlers, String> {
    let mut input_pins: GpioInputPins = HashMap::new();
    let mut output_pins: GpioOutputPins = HashMap::new();
    for pin in &config.pins {
        match pin.mode {
            GpioMode::INPUT => {
                let input: SysFsGpioInput = gpio::sysfs::SysFsGpioInput::open(pin.number).map_err(|err| format!("Could not open gpio {} on input mode: {}", pin.number, err))?;
                input_pins.insert(pin.number, input);
            },
            GpioMode::OUTPUT => {
                let output: SysFsGpioOutput = gpio::sysfs::SysFsGpioOutput::open(pin.number).map_err(|err| format!("Could not open gpio {} on output mode: {}", pin.number, err))?;
                output_pins.insert(pin.number, output);
            },
        }
    }
    Ok(GpioHandlers{inputs: input_pins, _outputs: output_pins})
}

#[get("/")]
async fn gpio_read(info: web::Query<PinRequest>, data: Data<Arc<Mutex<GpioHandlers>>>) -> impl Responder {
    let data: &mut GpioHandlers = &mut *data.lock().unwrap();
    let pin_handler: Option<&mut SysFsGpioInput> = data.inputs.get_mut(&info.id);
    if let Some(pin_handler) = pin_handler {
        let value: Result<GpioValue, std::io::Error> = pin_handler.read_value();
        match value {
            Ok(value) => HttpResponse::Ok().json(json!({"Value": value as u8})),
            Err(err) => HttpResponse::InternalServerError().json(json!({"Error": format!("Could not read pin {})", err)}))
        }
    } else {
        HttpResponse::NotFound().json(json!({"Error": "GPIO pin not found for read"}))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let gpio_config: GpioConfig = read_config_from_file("config.json").expect("Could not read config from file: {err}");
    println!("{gpio_config:?}");
    let gpio_handlers: GpioHandlers = prepare_gpio(&gpio_config).expect("Error preparing gpio");

    let thread_safe_gpio_handlers = Arc::new(Mutex::new(gpio_handlers));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(thread_safe_gpio_handlers.clone()))
            .service(gpio_read)
    })
    .bind(("127.0.0.1", 5679))?
    .run()
    .await?;
    Ok(())
}
