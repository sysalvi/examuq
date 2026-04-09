<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\Exam;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class MonitoringController extends Controller
{
    public function sessionsByExam(Request $request, int $examId): JsonResponse
    {
        $request->validate([
            'status' => ['nullable', 'string', 'max:32'],
            'per_page' => ['nullable', 'integer', 'min:1', 'max:100'],
        ]);

        $exam = Exam::query()->findOrFail($examId);

        $query = $exam->sessions()->orderByDesc('started_at');

        if ($request->filled('status')) {
            $query->where('status', $request->string('status')->toString());
        }

        $sessions = $query->paginate((int) $request->input('per_page', 20));

        return response()->json([
            'exam' => [
                'id' => $exam->id,
                'title' => $exam->title,
                'class_room' => $exam->class_room,
            ],
            'sessions' => $sessions,
        ]);
    }

}
