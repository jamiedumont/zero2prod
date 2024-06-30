use actix_web::HttpResponse;

pub async fn health_check() -> HttpResponse {
	// Return 200 response
	HttpResponse::Ok().finish()
}