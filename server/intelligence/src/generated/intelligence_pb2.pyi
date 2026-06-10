import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class IntentRequest(_message.Message):
    __slots__ = ("prompt", "user_id")
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    USER_ID_FIELD_NUMBER: _ClassVar[int]
    prompt: str
    user_id: str
    def __init__(self, prompt: _Optional[str] = ..., user_id: _Optional[str] = ...) -> None: ...

class IntentResponse(_message.Message):
    __slots__ = ("goal", "diet", "budget_limit", "budget_currency", "timeline", "constraints", "raw_analysis_json")
    GOAL_FIELD_NUMBER: _ClassVar[int]
    DIET_FIELD_NUMBER: _ClassVar[int]
    BUDGET_LIMIT_FIELD_NUMBER: _ClassVar[int]
    BUDGET_CURRENCY_FIELD_NUMBER: _ClassVar[int]
    TIMELINE_FIELD_NUMBER: _ClassVar[int]
    CONSTRAINTS_FIELD_NUMBER: _ClassVar[int]
    RAW_ANALYSIS_JSON_FIELD_NUMBER: _ClassVar[int]
    goal: str
    diet: str
    budget_limit: float
    budget_currency: str
    timeline: str
    constraints: _containers.RepeatedScalarFieldContainer[str]
    raw_analysis_json: str
    def __init__(self, goal: _Optional[str] = ..., diet: _Optional[str] = ..., budget_limit: _Optional[float] = ..., budget_currency: _Optional[str] = ..., timeline: _Optional[str] = ..., constraints: _Optional[_Iterable[str]] = ..., raw_analysis_json: _Optional[str] = ...) -> None: ...

class UserContext(_message.Message):
    __slots__ = ("user_id", "allergies", "medical_conditions", "preferred_foods", "avoided_foods", "daily_calorie_target", "target_protein_g", "target_carbs_g", "target_fat_g", "budget_limit", "preferred_cuisine")
    USER_ID_FIELD_NUMBER: _ClassVar[int]
    ALLERGIES_FIELD_NUMBER: _ClassVar[int]
    MEDICAL_CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    PREFERRED_FOODS_FIELD_NUMBER: _ClassVar[int]
    AVOIDED_FOODS_FIELD_NUMBER: _ClassVar[int]
    DAILY_CALORIE_TARGET_FIELD_NUMBER: _ClassVar[int]
    TARGET_PROTEIN_G_FIELD_NUMBER: _ClassVar[int]
    TARGET_CARBS_G_FIELD_NUMBER: _ClassVar[int]
    TARGET_FAT_G_FIELD_NUMBER: _ClassVar[int]
    BUDGET_LIMIT_FIELD_NUMBER: _ClassVar[int]
    PREFERRED_CUISINE_FIELD_NUMBER: _ClassVar[int]
    user_id: str
    allergies: _containers.RepeatedScalarFieldContainer[str]
    medical_conditions: _containers.RepeatedScalarFieldContainer[str]
    preferred_foods: _containers.RepeatedScalarFieldContainer[str]
    avoided_foods: _containers.RepeatedScalarFieldContainer[str]
    daily_calorie_target: float
    target_protein_g: float
    target_carbs_g: float
    target_fat_g: float
    budget_limit: float
    preferred_cuisine: str
    def __init__(self, user_id: _Optional[str] = ..., allergies: _Optional[_Iterable[str]] = ..., medical_conditions: _Optional[_Iterable[str]] = ..., preferred_foods: _Optional[_Iterable[str]] = ..., avoided_foods: _Optional[_Iterable[str]] = ..., daily_calorie_target: _Optional[float] = ..., target_protein_g: _Optional[float] = ..., target_carbs_g: _Optional[float] = ..., target_fat_g: _Optional[float] = ..., budget_limit: _Optional[float] = ..., preferred_cuisine: _Optional[str] = ...) -> None: ...

class OrchestrateGraphRequest(_message.Message):
    __slots__ = ("trace_id", "session_id", "prompt", "context", "history")
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_FIELD_NUMBER: _ClassVar[int]
    HISTORY_FIELD_NUMBER: _ClassVar[int]
    trace_id: str
    session_id: str
    prompt: str
    context: UserContext
    history: _containers.RepeatedCompositeFieldContainer[ChatMessageHistory]
    def __init__(self, trace_id: _Optional[str] = ..., session_id: _Optional[str] = ..., prompt: _Optional[str] = ..., context: _Optional[_Union[UserContext, _Mapping]] = ..., history: _Optional[_Iterable[_Union[ChatMessageHistory, _Mapping]]] = ...) -> None: ...

class ChatMessageHistory(_message.Message):
    __slots__ = ("role", "content", "sent_at")
    ROLE_FIELD_NUMBER: _ClassVar[int]
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    SENT_AT_FIELD_NUMBER: _ClassVar[int]
    role: str
    content: str
    sent_at: _timestamp_pb2.Timestamp
    def __init__(self, role: _Optional[str] = ..., content: _Optional[str] = ..., sent_at: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ...) -> None: ...

class AgentStepUpdate(_message.Message):
    __slots__ = ("step_id", "agent_name", "status", "step_type", "input_payload_json", "output_payload_json", "latency_ms", "error_message")
    STEP_ID_FIELD_NUMBER: _ClassVar[int]
    AGENT_NAME_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    STEP_TYPE_FIELD_NUMBER: _ClassVar[int]
    INPUT_PAYLOAD_JSON_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_PAYLOAD_JSON_FIELD_NUMBER: _ClassVar[int]
    LATENCY_MS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    step_id: str
    agent_name: str
    status: str
    step_type: str
    input_payload_json: str
    output_payload_json: str
    latency_ms: int
    error_message: str
    def __init__(self, step_id: _Optional[str] = ..., agent_name: _Optional[str] = ..., status: _Optional[str] = ..., step_type: _Optional[str] = ..., input_payload_json: _Optional[str] = ..., output_payload_json: _Optional[str] = ..., latency_ms: _Optional[int] = ..., error_message: _Optional[str] = ...) -> None: ...

class UIElement(_message.Message):
    __slots__ = ("type", "title", "config_json", "data_json")
    TYPE_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    CONFIG_JSON_FIELD_NUMBER: _ClassVar[int]
    DATA_JSON_FIELD_NUMBER: _ClassVar[int]
    type: str
    title: str
    config_json: str
    data_json: str
    def __init__(self, type: _Optional[str] = ..., title: _Optional[str] = ..., config_json: _Optional[str] = ..., data_json: _Optional[str] = ...) -> None: ...

class TextDelta(_message.Message):
    __slots__ = ("content", "delta_index", "is_complete", "delta_type")
    CONTENT_FIELD_NUMBER: _ClassVar[int]
    DELTA_INDEX_FIELD_NUMBER: _ClassVar[int]
    IS_COMPLETE_FIELD_NUMBER: _ClassVar[int]
    DELTA_TYPE_FIELD_NUMBER: _ClassVar[int]
    content: str
    delta_index: int
    is_complete: bool
    delta_type: str
    def __init__(self, content: _Optional[str] = ..., delta_index: _Optional[int] = ..., is_complete: _Optional[bool] = ..., delta_type: _Optional[str] = ...) -> None: ...

class OrchestrateGraphResponse(_message.Message):
    __slots__ = ("trace_id", "step_update", "final_text", "ui_element", "text_delta")
    TRACE_ID_FIELD_NUMBER: _ClassVar[int]
    STEP_UPDATE_FIELD_NUMBER: _ClassVar[int]
    FINAL_TEXT_FIELD_NUMBER: _ClassVar[int]
    UI_ELEMENT_FIELD_NUMBER: _ClassVar[int]
    TEXT_DELTA_FIELD_NUMBER: _ClassVar[int]
    trace_id: str
    step_update: AgentStepUpdate
    final_text: str
    ui_element: UIElement
    text_delta: TextDelta
    def __init__(self, trace_id: _Optional[str] = ..., step_update: _Optional[_Union[AgentStepUpdate, _Mapping]] = ..., final_text: _Optional[str] = ..., ui_element: _Optional[_Union[UIElement, _Mapping]] = ..., text_delta: _Optional[_Union[TextDelta, _Mapping]] = ...) -> None: ...

class ReflectionRequest(_message.Message):
    __slots__ = ("user_id", "session_id", "chat_message_id", "feedback_rating", "feedback_text")
    USER_ID_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    CHAT_MESSAGE_ID_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_RATING_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_TEXT_FIELD_NUMBER: _ClassVar[int]
    user_id: str
    session_id: str
    chat_message_id: str
    feedback_rating: int
    feedback_text: str
    def __init__(self, user_id: _Optional[str] = ..., session_id: _Optional[str] = ..., chat_message_id: _Optional[str] = ..., feedback_rating: _Optional[int] = ..., feedback_text: _Optional[str] = ...) -> None: ...

class ReflectionResponse(_message.Message):
    __slots__ = ("success", "extracted_memories")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    EXTRACTED_MEMORIES_FIELD_NUMBER: _ClassVar[int]
    success: bool
    extracted_memories: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, success: _Optional[bool] = ..., extracted_memories: _Optional[_Iterable[str]] = ...) -> None: ...
