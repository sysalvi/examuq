<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Exam;
use App\Models\ExamSession;
use Carbon\Carbon;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class SessionController extends Controller
{
    public function start(Request $request): JsonResponse
    {
        $payload = $request->validate([
            'exam_id' => ['required', 'integer', 'exists:exams,id'],
            'display_name' => ['required', 'string', 'max:255'],
            'class_room' => ['required', 'string', 'max:255'],
            'client_type' => ['required', 'string', 'in:desktop_client,chrome_extension'],
            'device_id' => ['nullable', 'string', 'max:255'],
        ]);

        $exam = Exam::query()->findOrFail($payload['exam_id']);

        if (! $exam->is_active) {
            return response()->json([
                'message' => 'Ujian tidak aktif.',
            ], 422);
        }

        $now = Carbon::now();

        $session = ExamSession::query()->create([
            'exam_id' => $exam->id,
            'display_name' => $payload['display_name'],
            'class_room' => $payload['class_room'],
            'client_type' => $payload['client_type'],
            'device_id' => $payload['device_id'] ?? null,
            'status' => 'active',
            'started_at' => $now,
            'last_heartbeat_at' => $now,
            'ip_address' => $request->ip(),
            'user_agent' => $request->userAgent(),
        ]);

        return response()->json([
            'message' => 'Sesi ujian dimulai.',
            'session_id' => $session->id,
        ], 201);
    }

    public function heartbeat(int $sessionId): JsonResponse
    {
        $session = ExamSession::query()
            ->select(['id', 'status', 'last_heartbeat_at'])
            ->findOrFail($sessionId);

        if ($session->status === 'finished') {
            return response()->json(null, 204);
        }

        $now = Carbon::now();
        $lastHeartbeatAt = $session->last_heartbeat_at;

        $shouldPersist =
            ! $lastHeartbeatAt
            || $session->status !== 'active'
            || $lastHeartbeatAt->diffInSeconds($now) >= 90;

        if ($shouldPersist) {
            $session->update([
                'last_heartbeat_at' => $now,
                'status' => 'active',
            ]);
        }

        return response()->json(null, 204);
    }

    public function end(Request $request, int $sessionId): JsonResponse
    {
        $request->validate([
            'reason' => ['nullable', 'string', 'max:255'],
        ]);

        $session = ExamSession::query()->findOrFail($sessionId);
        $now = Carbon::now();

        $session->update([
            'status' => 'finished',
            'ended_at' => $now,
            'last_heartbeat_at' => $now,
        ]);

        return response()->json([
            'message' => 'Sesi ditutup.',
            'session_id' => $session->id,
        ]);
    }
}
