use yew::prelude::*;
use crate::models::ParticipantRating;

#[derive(Properties, Clone, PartialEq)]
pub struct RatingDisplayProps {
    pub rating: Option<ParticipantRating>,
}

#[function_component(RatingDisplay)]
pub fn rating_display(props: &RatingDisplayProps) -> Html {
    let RatingDisplayProps { rating } = props;

    match rating {
        Some(rating) => {
            let trust_score = rating.trust_score.parse::<f32>().unwrap_or(0.0);
            let satisfaction_score = rating.satisfaction_score.parse::<f32>().unwrap_or(0.0);

            html! {
                <div class="flex items-center gap-2">
                    <div class="flex flex-col">
                        <div class="flex items-center">
                            <span class="text-sm text-gray-600 mr-2">{"Trust:"}</span>
                            <div class="flex">
                                {(0..5).map(|i| {
                                    let filled = i as f32 <= trust_score;
                                    html! {
                                        <svg class={if filled { "text-yellow-400" } else { "text-gray-300" }}
                                             viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
                                            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                        </svg>
                                    }
                                }).collect::<Html>()}
                            </div>
                        </div>
                        <div class="flex items-center">
                            <span class="text-sm text-gray-600 mr-2">{"Satisfaction:"}</span>
                            <div class="flex">
                                {(0..5).map(|i| {
                                    let filled = i as f32 <= satisfaction_score;
                                    html! {
                                        <svg class={if filled { "text-yellow-400" } else { "text-gray-300" }}
                                             viewBox="0 0 20 20" fill="currentColor" width="16" height="16">
                                            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                                        </svg>
                                    }
                                }).collect::<Html>()}
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
        None => html! {
            <div class="text-sm text-gray-400">{"No ratings yet"}</div>
        }
    }
}